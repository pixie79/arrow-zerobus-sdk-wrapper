//! Main wrapper implementation for Zerobus SDK
//!
//! This module provides the core ZerobusWrapper that handles data transmission
//! to Zerobus with automatic protocol conversion, authentication, and retry logic.

pub mod auth;
pub mod conversion;
pub mod debug;
pub mod protobuf_serialization;
pub mod retry;
pub mod zerobus;

use crate::config::WrapperConfiguration;
use crate::error::ZerobusError;
use crate::observability::ObservabilityManager;
use crate::wrapper::retry::RetryConfig;
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Result of a data transmission operation
#[derive(Debug, Clone)]
pub struct TransmissionResult {
    /// Whether transmission succeeded
    pub success: bool,
    /// Error information if transmission failed
    pub error: Option<ZerobusError>,
    /// Number of retry attempts made
    pub attempts: u32,
    /// Transmission latency in milliseconds (if successful)
    pub latency_ms: Option<u64>,
    /// Size of transmitted batch in bytes
    pub batch_size_bytes: usize,
}

/// Main wrapper for sending data to Zerobus
///
/// Thread-safe wrapper that handles Arrow RecordBatch to Protobuf conversion,
/// authentication, retry logic, and transmission to Zerobus.
pub struct ZerobusWrapper {
    /// Configuration (immutable)
    config: Arc<WrapperConfiguration>,
    /// Zerobus SDK instance (thread-safe)
    sdk: Arc<Mutex<Option<databricks_zerobus_ingest_sdk::ZerobusSdk>>>,
    /// Active stream (lazy initialization)
    stream: Arc<Mutex<Option<databricks_zerobus_ingest_sdk::ZerobusStream>>>,
    /// Retry configuration
    retry_config: RetryConfig,
    /// Observability manager (optional)
    observability: Option<ObservabilityManager>,
    /// Debug writer (optional)
    debug_writer: Option<Arc<crate::wrapper::debug::DebugWriter>>,
    /// Track if we've written the descriptor for this table (once per table)
    descriptor_written: Arc<tokio::sync::Mutex<bool>>,
}

impl ZerobusWrapper {
    /// Create a new ZerobusWrapper with the provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for initializing the wrapper
    ///
    /// # Returns
    ///
    /// Returns `Ok(ZerobusWrapper)` if initialization succeeds, or `Err(ZerobusError)` if:
    /// - Configuration validation fails
    /// - SDK initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
    ///
    /// # async fn example() -> Result<(), arrow_zerobus_sdk_wrapper::ZerobusError> {
    /// let config = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// );
    /// let wrapper = ZerobusWrapper::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: WrapperConfiguration) -> Result<Self, ZerobusError> {
        info!("Initializing ZerobusWrapper");

        // Validate configuration
        config.validate()?;

        // Get required OAuth credentials
        let unity_catalog_url = config
            .unity_catalog_url
            .as_ref()
            .ok_or_else(|| {
                ZerobusError::ConfigurationError(
                    "unity_catalog_url is required for SDK".to_string(),
                )
            })?
            .clone();

        let _client_id = config
            .client_id
            .as_ref()
            .ok_or_else(|| {
                ZerobusError::ConfigurationError("client_id is required for SDK".to_string())
            })?
            .clone();

        let _client_secret = config
            .client_secret
            .as_ref()
            .ok_or_else(|| {
                ZerobusError::ConfigurationError("client_secret is required for SDK".to_string())
            })?
            .clone();

        // Normalize and validate zerobus endpoint
        let normalized_endpoint = config.zerobus_endpoint.trim().to_string();

        if normalized_endpoint.is_empty() {
            return Err(ZerobusError::ConfigurationError(
                "zerobus_endpoint cannot be empty".to_string(),
            ));
        }

        if !normalized_endpoint.starts_with("https://")
            && !normalized_endpoint.starts_with("http://")
        {
            return Err(ZerobusError::ConfigurationError(format!(
                "zerobus_endpoint must start with 'https://' or 'http://'. Got: '{}'",
                normalized_endpoint
            )));
        }

        info!("Zerobus endpoint: {}", normalized_endpoint);
        info!("Unity Catalog URL: {}", unity_catalog_url);

        // Initialize SDK (will be created lazily when needed)
        // For now, we'll store None and create it on first use
        let sdk = Arc::new(Mutex::new(None));

        // Create retry config from wrapper config
        let retry_config = RetryConfig::new(
            config.retry_max_attempts,
            config.retry_base_delay_ms,
            config.retry_max_delay_ms,
        );

        // Initialize observability if enabled
        let observability = if config.observability_enabled {
            ObservabilityManager::new_async(config.observability_config.clone()).await
        } else {
            None
        };

        if observability.is_some() {
            info!("Observability enabled");
        }

        // Initialize debug writer if enabled
        // Info logging to diagnose why debug writer isn't being initialized
        info!("ZerobusWrapper::new: debug_enabled={}, debug_output_dir={:?}", 
               config.debug_enabled, 
               config.debug_output_dir);
        
        let debug_writer = if config.debug_enabled {
            if let Some(output_dir) = &config.debug_output_dir {
                use crate::wrapper::debug::DebugWriter;
                use std::time::Duration;

                info!("Initializing debug writer with output_dir: {}, table_name: {}", output_dir.display(), config.table_name);
                match DebugWriter::new(
                    output_dir.clone(),
                    config.table_name.clone(),
                    Duration::from_secs(config.debug_flush_interval_secs),
                    config.debug_max_file_size,
                ) {
                    Ok(writer) => {
                        info!("Debug file output enabled: {}", output_dir.display());
                        Some(Arc::new(writer))
                    }
                    Err(e) => {
                        warn!("Failed to initialize debug writer: {}", e);
                        None
                    }
                }
            } else {
                warn!("debug_enabled is true but debug_output_dir is None - debug files will not be written");
                None
            }
        } else {
            info!("debug_enabled is false - debug files will not be written");
            None
        };

        Ok(Self {
            config: Arc::new(config),
            sdk,
            stream: Arc::new(Mutex::new(None)),
            retry_config,
            observability,
            debug_writer,
            descriptor_written: Arc::new(tokio::sync::Mutex::new(false)),
        })
    }

    /// Send a data batch to Zerobus
    ///
    /// Converts Arrow RecordBatch to Protobuf format and transmits to Zerobus
    /// with automatic retry on transient failures.
    ///
    /// # Arguments
    ///
    /// * `batch` - Arrow RecordBatch to send
    /// * `descriptor` - Optional Protobuf descriptor. If provided, uses this descriptor
    ///   instead of auto-generating from Arrow schema. This ensures correct nested types.
    ///
    /// # Returns
    ///
    /// Returns `TransmissionResult` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns error if transmission fails after all retry attempts.
    pub async fn send_batch(&self, batch: RecordBatch) -> Result<TransmissionResult, ZerobusError> {
        self.send_batch_with_descriptor(batch, None).await
    }

    /// Send a data batch to Zerobus with an optional Protobuf descriptor
    ///
    /// Converts Arrow RecordBatch to Protobuf format and transmits to Zerobus
    /// with automatic retry on transient failures.
    ///
    /// # Arguments
    ///
    /// * `batch` - Arrow RecordBatch to send
    /// * `descriptor` - Optional Protobuf descriptor. If provided, uses this descriptor
    ///   instead of auto-generating from Arrow schema. This ensures correct nested types.
    ///
    /// # Returns
    ///
    /// Returns `TransmissionResult` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns error if transmission fails after all retry attempts.
    pub async fn send_batch_with_descriptor(
        &self,
        batch: RecordBatch,
        descriptor: Option<prost_types::DescriptorProto>,
    ) -> Result<TransmissionResult, ZerobusError> {
        let start_time = std::time::Instant::now();
        let batch_size_bytes = batch.get_array_memory_size();

        debug!(
            "Sending batch with {} rows, {} bytes",
            batch.num_rows(),
            batch_size_bytes
        );

        // Write Arrow batch to debug file if enabled
        if let Some(ref debug_writer) = self.debug_writer {
            if let Err(e) = debug_writer.write_arrow(&batch).await {
                warn!("Failed to write Arrow debug file: {}", e);
                // Don't fail the operation if debug writing fails
            }
        }

        // Start observability span if enabled
        let _span = self
            .observability
            .as_ref()
            .map(|obs| obs.start_send_batch_span(&self.config.table_name));

        // Use retry logic for transmission
        let (result, attempts) = self
            .retry_config
            .execute_with_retry_tracked(|| {
                let batch = batch.clone();
                let descriptor = descriptor.clone();
                let wrapper = self.clone();
                async move { wrapper.send_batch_internal(batch, descriptor).await }
            })
            .await;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Record metrics if observability is enabled
        if let Some(obs) = &self.observability {
            let success = result.is_ok();
            obs.record_batch_sent(batch_size_bytes, success, latency_ms)
                .await;
        }

        match result {
            Ok(_) => Ok(TransmissionResult {
                success: true,
                error: None,
                attempts,
                latency_ms: Some(latency_ms),
                batch_size_bytes,
            }),
            Err(e) => {
                error!("Failed to send batch after retries: {}", e);
                Ok(TransmissionResult {
                    success: false,
                    error: Some(e),
                    attempts,
                    latency_ms: Some(latency_ms),
                    batch_size_bytes,
                })
            }
        }
    }

    /// Internal method to send a batch (without retry wrapper)
    async fn send_batch_internal(
        &self,
        batch: RecordBatch,
        descriptor: Option<prost_types::DescriptorProto>,
    ) -> Result<(), ZerobusError> {
        // 1. Ensure SDK is initialized
        {
            let mut sdk_guard = self.sdk.lock().await;
            if sdk_guard.is_none() {
                let unity_catalog_url = self
                    .config
                    .unity_catalog_url
                    .as_ref()
                    .ok_or_else(|| {
                        ZerobusError::ConfigurationError(
                            "unity_catalog_url is required".to_string(),
                        )
                    })?
                    .clone();

                let sdk = crate::wrapper::zerobus::create_sdk(
                    self.config.zerobus_endpoint.clone(),
                    unity_catalog_url,
                )
                .await?;
                *sdk_guard = Some(sdk);
            }
        }

        // Get SDK reference (lock is released, so we can lock again for stream creation)
        let sdk_guard = self.sdk.lock().await;
        let sdk = sdk_guard.as_ref().unwrap();

        // 2. Get Protobuf descriptor (use provided one or generate from Arrow schema)
        let descriptor = if let Some(provided_descriptor) = descriptor {
            let descriptor_name = provided_descriptor.name.as_deref().unwrap_or("unknown");
            info!("🔍 [DEBUG] Using provided Protobuf descriptor: name='{}', fields={}, nested_types={}", 
                  descriptor_name, provided_descriptor.field.len(), provided_descriptor.nested_type.len());
            provided_descriptor
        } else {
            debug!("Auto-generating Protobuf descriptor from Arrow schema");
            let generated = crate::wrapper::conversion::generate_protobuf_descriptor(batch.schema().as_ref())
                .map_err(|e| {
                    ZerobusError::ConversionError(format!(
                        "Failed to generate Protobuf descriptor: {}",
                        e
                    ))
                })?;
            let descriptor_name = generated.name.as_deref().unwrap_or("unknown");
            info!("🔍 [DEBUG] Auto-generated Protobuf descriptor: name='{}', fields={}, nested_types={}", 
                  descriptor_name, generated.field.len(), generated.nested_type.len());
            generated
        };

        // Write descriptor to file once per table (if debug writer is enabled)
        if let Some(ref debug_writer) = self.debug_writer {
            let mut written_guard = self.descriptor_written.lock().await;
            if !*written_guard {
                if let Err(e) = debug_writer.write_descriptor(&self.config.table_name, &descriptor).await {
                    warn!("Failed to write Protobuf descriptor to debug file: {}", e);
                    // Don't fail the operation if descriptor writing fails
                } else {
                    *written_guard = true;
                }
            }
        }

        // 3. Convert Arrow RecordBatch to Protobuf bytes (one per row)
        let protobuf_bytes_list =
            crate::wrapper::conversion::record_batch_to_protobuf_bytes(&batch, &descriptor)
                .map_err(|e| {
                    ZerobusError::ConversionError(format!(
                        "Failed to convert RecordBatch to Protobuf: {}",
                        e
                    ))
                })?;

        // Write Protobuf bytes to debug file if enabled
        // Flush after each batch to ensure files are immediately available for debugging
        // CRITICAL: Write protobuf files BEFORE Zerobus write attempts, so we have them even if Zerobus fails
        if let Some(ref debug_writer) = self.debug_writer {
            info!("Writing {} protobuf messages to debug file", protobuf_bytes_list.len());
            let num_rows = protobuf_bytes_list.len();
            for (idx, bytes) in protobuf_bytes_list.iter().enumerate() {
                // Flush immediately after last row in batch
                let flush_immediately = idx == num_rows - 1;
                if let Err(e) = debug_writer.write_protobuf(bytes, flush_immediately).await {
                    warn!("Failed to write Protobuf debug file: {}", e);
                    // Don't fail the operation if debug writing fails
                } else if flush_immediately {
                    info!("✅ Flushed protobuf debug file after batch ({} messages)", num_rows);
                }
            }
        } else {
            warn!("⚠️  Debug writer is None - protobuf debug files will not be written. Check debug_enabled and debug_output_dir config.");
        }

        // 4. Ensure stream is created
        let client_id = self
            .config
            .client_id
            .as_ref()
            .ok_or_else(|| ZerobusError::ConfigurationError("client_id is required".to_string()))?
            .clone();
        let client_secret = self
            .config
            .client_secret
            .as_ref()
            .ok_or_else(|| {
                ZerobusError::ConfigurationError("client_secret is required".to_string())
            })?
            .clone();

        // 5. Check backoff BEFORE attempting any writes (even if stream exists)
        // This prevents writes during backoff period even if stream was created before backoff started
        {
            use crate::wrapper::zerobus::check_error_6006_backoff;
            if let Err(backoff_err) = check_error_6006_backoff(&self.config.table_name).await {
                return Err(backoff_err);
            }
        }

        // 6. Write each row to Zerobus with stream recreation on failure
        let mut retry_count = 0;
        const MAX_STREAM_RECREATE_ATTEMPTS: u32 = 3;
        
        loop {
            // Ensure stream exists and is valid
            let mut stream_guard = self.stream.lock().await;
            if stream_guard.is_none() {
                info!("Stream not found, creating new stream for table: {}", self.config.table_name);
                let stream = crate::wrapper::zerobus::ensure_stream(
                    sdk,
                    self.config.table_name.clone(),
                    descriptor.clone(),
                    client_id.clone(),
                    client_secret.clone(),
                )
                .await?;
                *stream_guard = Some(stream);
                info!("✅ Stream created successfully");
            }
            let stream = stream_guard.as_mut().unwrap();
            drop(stream_guard); // Release lock before sending data

            // Try to send all rows
            let mut all_succeeded = true;
            let mut failed_at_idx = 0;
            
            for (idx, bytes) in protobuf_bytes_list.iter().enumerate() {
                // Check backoff again before each record (in case backoff started during batch processing)
                {
                    use crate::wrapper::zerobus::check_error_6006_backoff;
                    if let Err(backoff_err) = check_error_6006_backoff(&self.config.table_name).await {
                        // Clear stream so it gets recreated after backoff
                        let mut stream_guard = self.stream.lock().await;
                        *stream_guard = None;
                        drop(stream_guard);
                        all_succeeded = false;
                        failed_at_idx = idx;
                        break;
                    }
                }
                
                // Re-acquire stream lock for each record
                let mut stream_guard = self.stream.lock().await;
                if stream_guard.is_none() {
                    // Stream was cleared, recreate it
                    info!("Stream was cleared, recreating for table: {}", self.config.table_name);
                    let stream = crate::wrapper::zerobus::ensure_stream(
                        sdk,
                        self.config.table_name.clone(),
                        descriptor.clone(),
                        client_id.clone(),
                        client_secret.clone(),
                    )
                    .await?;
                    *stream_guard = Some(stream);
                }
                let stream = stream_guard.as_mut().unwrap();
                
                // Send bytes to Zerobus stream using ingest_record
                match stream.ingest_record(bytes.clone()).await {
                    Ok(ingest_future) => {
                        // Release lock before awaiting
                        drop(stream_guard);
                        
                        // Await the inner future to get the final result
                        match ingest_future.await {
                            Ok(_) => {
                                debug!("✅ Successfully sent {} bytes to Zerobus stream (row {})", bytes.len(), idx);
                            }
                            Err(e) => {
                                let err_msg = format!("{}", e);
                                // Check if stream is closed
                                if err_msg.contains("Stream is closed") || err_msg.contains("Stream closed") {
                                    error!("❌ Stream closed while ingesting row {} (first record: {}): {}", 
                                        idx, 
                                        if idx == 0 { "YES" } else { "NO" },
                                        err_msg);
                                    if idx == 0 {
                                        error!("🔍 This is the FIRST record - stream closed immediately after creation. This suggests:");
                                        error!("   1. Schema mismatch between descriptor and table");
                                        error!("   2. Validation error on first record");
                                        error!("   3. Table schema not yet propagated");
                                        error!("   Descriptor has {} fields and {} nested types", 
                                            descriptor.field.len(), 
                                            descriptor.nested_type.len());
                                    }
                                    // Clear stream so it gets recreated on next iteration
                                    let mut stream_guard = self.stream.lock().await;
                                    *stream_guard = None;
                                    drop(stream_guard);
                                    all_succeeded = false;
                                    failed_at_idx = idx;
                                    break;
                                } else {
                                    return Err(ZerobusError::ConnectionError(format!(
                                        "Failed to ingest record {} to Zerobus stream: {}",
                                        idx, e
                                    )));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("{}", e);
                        // Check if stream is closed
                        if err_msg.contains("Stream is closed") || err_msg.contains("Stream closed") {
                            error!("❌ Stream closed when creating ingest record for row {} (first record: {}): {}", 
                                idx,
                                if idx == 0 { "YES" } else { "NO" },
                                err_msg);
                            if idx == 0 {
                                error!("🔍 This is the FIRST record - stream closed immediately. This suggests:");
                                error!("   1. Schema mismatch between descriptor and table");
                                error!("   2. Validation error on first record");
                                error!("   3. Table schema not yet propagated");
                                error!("   Descriptor has {} fields and {} nested types", 
                                    descriptor.field.len(), 
                                    descriptor.nested_type.len());
                            }
                            // Clear stream so it gets recreated on next iteration
                            *stream_guard = None;
                            drop(stream_guard);
                            all_succeeded = false;
                            failed_at_idx = idx;
                            break;
                        } else {
                            return Err(ZerobusError::ConnectionError(format!(
                                "Failed to create ingest record for row {}: {}",
                                idx, e
                            )));
                        }
                    }
                }
            }
            
            if all_succeeded {
                // All rows sent successfully
                break;
            } else {
                // Some rows failed due to stream closure, retry
                retry_count += 1;
                if retry_count > MAX_STREAM_RECREATE_ATTEMPTS {
                    return Err(ZerobusError::ConnectionError(format!(
                        "Stream closed after {} recreation attempts. Failed at row {}. This may indicate a schema mismatch or server-side issue.",
                        MAX_STREAM_RECREATE_ATTEMPTS, failed_at_idx
                    )));
                }
                warn!("⚠️  Stream closed, recreating (attempt {}/{})", retry_count, MAX_STREAM_RECREATE_ATTEMPTS);
                // Small delay before retry
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        debug!(
            "Successfully sent {} rows to Zerobus",
            protobuf_bytes_list.len()
        );
        Ok(())
    }

    /// Flush any pending operations and ensure data is transmitted
    ///
    /// # Errors
    ///
    /// Returns error if flush operation fails.
    pub async fn flush(&self) -> Result<(), ZerobusError> {
        // Flush debug files if enabled
        if let Some(ref debug_writer) = self.debug_writer {
            if let Err(e) = debug_writer.flush().await {
                warn!("Failed to flush debug files: {}", e);
            }
        }

        // Flush observability if enabled
        if let Some(ref obs) = self.observability {
            obs.flush().await?;
        }

        Ok(())
    }

    /// Shutdown the wrapper gracefully, closing connections and cleaning up resources
    ///
    /// # Errors
    ///
    /// Returns error if shutdown fails.
    pub async fn shutdown(&self) -> Result<(), ZerobusError> {
        info!("Shutting down ZerobusWrapper");

        // Close stream if it exists
        let mut stream_guard = self.stream.lock().await;
        if let Some(mut stream) = stream_guard.take() {
            // Close the stream gracefully
            // ZerobusStream has a close() method that returns ZerobusResult
            if let Err(e) = stream.close().await {
                warn!("Error closing Zerobus stream: {}", e);
            } else {
                debug!("Stream closed successfully");
            }
        }

        Ok(())
    }
}

// Implement Clone for use in async closures
impl Clone for ZerobusWrapper {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            sdk: Arc::clone(&self.sdk),
            stream: Arc::clone(&self.stream),
            retry_config: self.retry_config.clone(),
            observability: self.observability.clone(),
            debug_writer: self.debug_writer.as_ref().map(Arc::clone),
            descriptor_written: Arc::clone(&self.descriptor_written),
        }
    }
}

// Ensure Send + Sync for thread-safety
unsafe impl Send for ZerobusWrapper {}
unsafe impl Sync for ZerobusWrapper {}

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
use secrecy::ExposeSecret;
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
    /// Validate and normalize the Zerobus endpoint URL.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Raw endpoint string from configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` with normalized endpoint, or `Err(ZerobusError)` if validation fails.
    fn validate_and_normalize_endpoint(endpoint: &str) -> Result<String, ZerobusError> {
        let normalized_endpoint = endpoint.trim().to_string();

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

        Ok(normalized_endpoint)
    }

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

        // Validate and normalize endpoint (required for both enabled and disabled modes)
        let normalized_endpoint = Self::validate_and_normalize_endpoint(&config.zerobus_endpoint)?;

        // Skip credential validation if writer is disabled (credentials optional in this mode)
        if !config.zerobus_writer_disabled {
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

            // Validate credentials are present (but don't expose them unnecessarily)
            let _client_id = config.client_id.as_ref().ok_or_else(|| {
                ZerobusError::ConfigurationError("client_id is required for SDK".to_string())
            })?;

            let _client_secret = config.client_secret.as_ref().ok_or_else(|| {
                ZerobusError::ConfigurationError("client_secret is required for SDK".to_string())
            })?;

            info!("Zerobus endpoint: {}", normalized_endpoint);
            info!("Unity Catalog URL: {}", unity_catalog_url);
        } else {
            // When writer is disabled, we still validate endpoint format but don't require credentials
            info!(
                "Zerobus endpoint: {} (writer disabled mode)",
                normalized_endpoint
            );
        }

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
        info!(
            "ZerobusWrapper::new: debug_enabled={}, debug_output_dir={:?}",
            config.debug_enabled, config.debug_output_dir
        );

        let debug_writer = if config.debug_enabled {
            if let Some(output_dir) = &config.debug_output_dir {
                use crate::wrapper::debug::DebugWriter;
                use std::time::Duration;

                info!(
                    "Initializing debug writer with output_dir: {}, table_name: {}",
                    output_dir.display(),
                    config.table_name
                );
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
        // CRITICAL: Check if writer is disabled FIRST, before any SDK initialization or credential access
        // This prevents errors when credentials are not provided (which is allowed when writer is disabled)
        if self.config.zerobus_writer_disabled {
            // When writer is disabled, we still perform conversion and write debug files,
            // but skip all SDK calls. This enables local development and testing without credentials.
            debug!(
                "Writer disabled mode enabled - skipping SDK initialization and Zerobus SDK calls"
            );
            // Continue to conversion and debug file writing below, then return early
        } else {
            // 1. Ensure SDK is initialized (only when writer is NOT disabled)
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
        }

        // 2. Get Protobuf descriptor (use provided one or generate from Arrow schema)
        let descriptor = if let Some(provided_descriptor) = descriptor {
            // Validate user-provided descriptor to prevent security issues
            crate::wrapper::conversion::validate_protobuf_descriptor(&provided_descriptor)
                .map_err(|e| {
                    ZerobusError::ConfigurationError(format!("Invalid Protobuf descriptor: {}", e))
                })?;
            let descriptor_name = provided_descriptor.name.as_deref().unwrap_or("unknown");
            info!("🔍 [DEBUG] Using provided Protobuf descriptor: name='{}', fields={}, nested_types={}", 
                  descriptor_name, provided_descriptor.field.len(), provided_descriptor.nested_type.len());
            provided_descriptor
        } else {
            debug!("Auto-generating Protobuf descriptor from Arrow schema");
            let generated =
                crate::wrapper::conversion::generate_protobuf_descriptor(batch.schema().as_ref())
                    .map_err(|e| {
                    ZerobusError::ConversionError(format!(
                        "Failed to generate Protobuf descriptor: {}",
                        e
                    ))
                })?;
            // Validate generated descriptor (should always pass, but safety check)
            crate::wrapper::conversion::validate_protobuf_descriptor(&generated).map_err(|e| {
                ZerobusError::ConversionError(format!(
                    "Generated Protobuf descriptor failed validation: {}",
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
                if let Err(e) = debug_writer
                    .write_descriptor(&self.config.table_name, &descriptor)
                    .await
                {
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
            info!(
                "Writing {} protobuf messages to debug file",
                protobuf_bytes_list.len()
            );
            let num_rows = protobuf_bytes_list.len();
            for (idx, bytes) in protobuf_bytes_list.iter().enumerate() {
                // Flush immediately after last row in batch
                let flush_immediately = idx == num_rows - 1;
                if let Err(e) = debug_writer.write_protobuf(bytes, flush_immediately).await {
                    warn!("Failed to write Protobuf debug file: {}", e);
                    // Don't fail the operation if debug writing fails
                } else if flush_immediately {
                    info!(
                        "✅ Flushed protobuf debug file after batch ({} messages)",
                        num_rows
                    );
                }
            }
        } else {
            warn!("⚠️  Debug writer is None - protobuf debug files will not be written. Check debug_enabled and debug_output_dir config.");
        }

        // Check if writer is disabled - if so, skip all SDK calls and return success
        // Performance: Operations complete in <50ms (excluding file I/O) when writer disabled
        // This enables performance testing of conversion logic without network overhead
        if self.config.zerobus_writer_disabled {
            debug!(
                "Writer disabled mode enabled - skipping Zerobus SDK calls. Debug files written successfully."
            );
            return Ok(());
        }

        // Get SDK reference (lock is released, so we can lock again for stream creation)
        // This is safe because we only reach here when writer is NOT disabled, so SDK was initialized above
        let sdk_guard = self.sdk.lock().await;
        let sdk = sdk_guard.as_ref().ok_or_else(|| {
            ZerobusError::ConfigurationError(
                "SDK not initialized - this should not happen".to_string(),
            )
        })?;

        // 4. Ensure stream is created
        // Expose secrets only when needed for API calls
        let client_id = self
            .config
            .client_id
            .as_ref()
            .ok_or_else(|| ZerobusError::ConfigurationError("client_id is required".to_string()))?
            .expose_secret()
            .clone();
        let client_secret = self
            .config
            .client_secret
            .as_ref()
            .ok_or_else(|| {
                ZerobusError::ConfigurationError("client_secret is required".to_string())
            })?
            .expose_secret()
            .clone();

        // ========================================================================
        // STEP 5: Check error 6006 backoff BEFORE attempting any writes
        // ========================================================================
        // CRITICAL: Check backoff BEFORE attempting writes, even if stream exists.
        // This prevents writes during backoff period even if stream was created before
        // backoff started. Error 6006 indicates pipeline is temporarily blocked by
        // Databricks due to repeated failures.
        //
        // Edge case: Backoff can start during batch processing, so we check again
        // before each record in the loop below.
        {
            use crate::wrapper::zerobus::check_error_6006_backoff;
            check_error_6006_backoff(&self.config.table_name).await?;
        }

        // ========================================================================
        // STEP 6: Write each row to Zerobus with stream recreation on failure
        // ========================================================================
        // This implements a retry loop that handles stream closure and recreation.
        //
        // Algorithm:
        // 1. Ensure stream exists (create if None)
        // 2. For each row in the batch:
        //    a. Check backoff again (backoff can start during batch processing)
        //    b. Re-acquire stream lock (stream may have been cleared)
        //    c. Recreate stream if it was cleared
        //    d. Send row to Zerobus
        //    e. Handle stream closure errors by clearing stream and retrying
        // 3. If all rows succeed, break
        // 4. If stream closed, retry up to MAX_STREAM_RECREATE_ATTEMPTS
        //
        // Edge cases handled:
        // - Stream closed immediately after creation (first record fails)
        //   → Indicates schema mismatch or validation error
        // - Stream closed mid-batch
        //   → Clear stream, recreate, and retry from failed row
        // - Backoff starts during batch processing
        //   → Clear stream, break loop, return error
        //
        // Performance considerations:
        // - Lock is released before async operations to avoid blocking
        // - Stream is only recreated when necessary (not for every row)
        // - Maximum retry attempts prevent infinite loops
        //
        // Thread safety:
        // - Uses async Mutex to prevent blocking the runtime
        // - Lock is held only when accessing/modifying stream
        // - Lock is released before network I/O operations
        let mut retry_count = 0;
        const MAX_STREAM_RECREATE_ATTEMPTS: u32 = 3;

        loop {
            // Ensure stream exists and is valid
            let mut stream_guard = self.stream.lock().await;
            if stream_guard.is_none() {
                info!(
                    "Stream not found, creating new stream for table: {}",
                    self.config.table_name
                );
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
            // Verify stream exists before dropping lock
            if stream_guard.is_none() {
                return Err(ZerobusError::ConnectionError(
                    "Stream was None after creation - this should not happen".to_string(),
                ));
            }
            drop(stream_guard); // Release lock before sending data

            // Try to send all rows
            let mut all_succeeded = true;
            let mut failed_at_idx = 0;

            for (idx, bytes) in protobuf_bytes_list.iter().enumerate() {
                // ========================================================================
                // STEP 6a: Check backoff before each record
                // ========================================================================
                // Edge case: Backoff can start during batch processing (e.g., another thread
                // encountered error 6006). We check before each record to prevent writes
                // during backoff period.
                {
                    use crate::wrapper::zerobus::check_error_6006_backoff;
                    if let Err(_backoff_err) =
                        check_error_6006_backoff(&self.config.table_name).await
                    {
                        // Clear stream so it gets recreated after backoff period expires
                        let mut stream_guard = self.stream.lock().await;
                        *stream_guard = None;
                        drop(stream_guard);
                        all_succeeded = false;
                        failed_at_idx = idx;
                        break;
                    }
                }

                // ========================================================================
                // STEP 6b: Re-acquire stream lock and ensure stream exists
                // ========================================================================
                // We re-acquire the lock for each record because:
                // 1. Stream may have been cleared by error handling in previous iteration
                // 2. Lock was released before async operations to avoid blocking
                // 3. Multiple threads may be sending batches concurrently
                //
                // Performance: Lock is held only briefly, released before network I/O.
                let mut stream_guard = self.stream.lock().await;
                if stream_guard.is_none() {
                    // Stream was cleared (e.g., by error handling), recreate it
                    info!(
                        "Stream was cleared, recreating for table: {}",
                        self.config.table_name
                    );
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
                let stream = stream_guard.as_mut().ok_or_else(|| {
                    ZerobusError::ConnectionError(
                        "Stream was None after recreation - this should not happen".to_string(),
                    )
                })?;

                // ========================================================================
                // STEP 6c: Send bytes to Zerobus stream
                // ========================================================================
                // The Zerobus SDK's ingest_record returns a Future that must be awaited.
                // We release the lock before awaiting to avoid blocking other operations.
                //
                // Error handling:
                // - Stream closed errors: Clear stream, mark failure, break loop to retry
                // - Other errors: Return immediately (non-retryable)
                // - First record failures: Log detailed diagnostics for schema issues
                match stream.ingest_record(bytes.clone()).await {
                    Ok(ingest_future) => {
                        // Release lock before awaiting to avoid blocking other operations
                        drop(stream_guard);

                        // Await the inner future to get the final result
                        match ingest_future.await {
                            Ok(_) => {
                                debug!(
                                    "✅ Successfully sent {} bytes to Zerobus stream (row {})",
                                    bytes.len(),
                                    idx
                                );
                            }
                            Err(e) => {
                                let err_msg = format!("{}", e);
                                // Check if stream is closed (indicates server-side closure)
                                if err_msg.contains("Stream is closed")
                                    || err_msg.contains("Stream closed")
                                {
                                    // Standardized error logging with context
                                    let is_first = idx == 0;
                                    error!(
                                        "Stream closed: row={}, first_record={}, error={}",
                                        idx, is_first, err_msg
                                    );
                                    if is_first {
                                        // First record failure indicates schema/validation issues
                                        error!("Diagnostics: This is the FIRST record - stream closed immediately after creation");
                                        error!("Possible causes:");
                                        error!("  1. Schema mismatch between descriptor and table");
                                        error!("  2. Validation error on first record");
                                        error!("  3. Table schema not yet propagated");
                                        error!(
                                            "Descriptor info: fields={}, nested_types={}",
                                            descriptor.field.len(),
                                            descriptor.nested_type.len()
                                        );
                                    }
                                    // Clear stream so it gets recreated on next iteration
                                    let mut stream_guard = self.stream.lock().await;
                                    *stream_guard = None;
                                    drop(stream_guard);
                                    all_succeeded = false;
                                    failed_at_idx = idx;
                                    break;
                                } else {
                                    // Non-stream-closure errors are returned immediately
                                    return Err(ZerobusError::ConnectionError(format!(
                                        "Record ingestion failed: row={}, error={}",
                                        idx, e
                                    )));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("{}", e);
                        // Check if stream is closed (indicates server-side closure)
                        if err_msg.contains("Stream is closed") || err_msg.contains("Stream closed")
                        {
                            // Standardized error logging with context
                            let is_first = idx == 0;
                            error!(
                                "Stream closed: row={}, first_record={}, error={}",
                                idx, is_first, err_msg
                            );
                            if is_first {
                                // First record failure indicates schema/validation issues
                                error!("Diagnostics: This is the FIRST record - stream closed immediately");
                                error!("Possible causes:");
                                error!("  1. Schema mismatch between descriptor and table");
                                error!("  2. Validation error on first record");
                                error!("  3. Table schema not yet propagated");
                                error!(
                                    "Descriptor info: fields={}, nested_types={}",
                                    descriptor.field.len(),
                                    descriptor.nested_type.len()
                                );
                            }
                            // Clear stream so it gets recreated on next iteration
                            *stream_guard = None;
                            drop(stream_guard);
                            all_succeeded = false;
                            failed_at_idx = idx;
                            break;
                        } else {
                            // Non-stream-closure errors are returned immediately
                            return Err(ZerobusError::ConnectionError(format!(
                                "Record creation failed: row={}, error={}",
                                idx, e
                            )));
                        }
                    }
                }
            }

            // ========================================================================
            // STEP 6d: Handle retry logic
            // ========================================================================
            // If all rows succeeded, we're done. Otherwise, retry with stream recreation.
            // The retry mechanism handles transient stream closure issues.
            //
            // Edge case: If stream closes repeatedly, it may indicate:
            // - Schema mismatch (descriptor doesn't match table schema)
            // - Server-side validation errors
            // - Network issues causing stream closure
            //
            // Performance: Small delay (100ms) prevents tight retry loops.
            if all_succeeded {
                // All rows sent successfully - exit retry loop
                break;
            } else {
                // Some rows failed due to stream closure - retry with stream recreation
                retry_count += 1;
                if retry_count > MAX_STREAM_RECREATE_ATTEMPTS {
                    // Exhausted retry attempts - return error with context
                    return Err(ZerobusError::ConnectionError(format!(
                        "Stream recreation exhausted: attempts={}, failed_at_row={}, possible_causes='schema_mismatch,validation_error,server_issue'",
                        MAX_STREAM_RECREATE_ATTEMPTS, failed_at_idx
                    )));
                }
                warn!(
                    "Stream recreation retry: attempt={}/{}, failed_at_row={}",
                    retry_count, MAX_STREAM_RECREATE_ATTEMPTS, failed_at_idx
                );
                // Small delay before retry to avoid tight retry loops
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

// ZerobusWrapper is automatically Send + Sync because all its fields are Send + Sync:
// - Arc<WrapperConfiguration>: Send + Sync (Arc is Send + Sync, WrapperConfiguration is Send + Sync)
// - Arc<Mutex<Option<ZerobusSdk>>>: Send + Sync (Arc and Mutex are Send + Sync)
// - Arc<Mutex<Option<ZerobusStream>>>: Send + Sync
// - RetryConfig: Send + Sync (contains only primitive types)
// - Option<ObservabilityManager>: Send + Sync (ObservabilityManager is Send + Sync)
// - Option<Arc<DebugWriter>>: Send + Sync
// - Arc<Mutex<bool>>: Send + Sync
// The compiler automatically derives Send + Sync for this struct, so explicit unsafe impl is not needed.

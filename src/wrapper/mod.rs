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

/// Internal result from send_batch_internal containing per-row error information
struct BatchTransmissionResult {
    /// Successful row indices
    successful_rows: Vec<usize>,
    /// Failed rows with errors
    failed_rows: Vec<(usize, ZerobusError)>,
}

/// Result of a data transmission operation
///
/// This struct provides comprehensive information about the result of sending a batch
/// to Zerobus, including per-row success/failure tracking and error details.
///
/// # Per-Row Error Tracking
///
/// The struct supports per-row error tracking, allowing identification of which
/// specific rows succeeded or failed during batch transmission. This enables:
///
/// - **Partial batch success**: Some rows can succeed while others fail
/// - **Quarantine workflows**: Extract and quarantine only failed rows
/// - **Error analysis**: Group errors by type, analyze patterns, track statistics
///
/// # Field Semantics
///
/// - **`success`**: `true` if ANY rows succeeded, `false` if ALL rows failed or batch-level error occurred
/// - **`error`**: Batch-level error (e.g., authentication failure, connection error before processing)
///   - `None` if no batch-level error occurred (even if some rows failed)
/// - **`failed_rows`**: Per-row failures
///   - `None` if all rows succeeded
///   - `Some(vec![])` if batch-level error only (no per-row processing occurred)
///   - `Some(vec![...])` for per-row failures
/// - **`successful_rows`**: Per-row successes
///   - `None` if all rows failed
///   - `Some(vec![])` if no rows succeeded
///   - `Some(vec![...])` for successful rows
/// - **`total_rows`**: Total number of rows in the batch (0 for empty batches)
/// - **`successful_count`**: Number of rows that succeeded (always equals `successful_rows.len()` if `Some`)
/// - **`failed_count`**: Number of rows that failed (always equals `failed_rows.len()` if `Some`)
///
/// # Edge Cases
///
/// - **Empty batch** (`total_rows == 0`): Returns `success=true`, `successful_count=0`, `failed_count=0`
/// - **Batch-level errors**: Authentication/connection errors before processing return `error=Some(...)`, `failed_rows=None`
/// - **All rows failed**: Returns `success=false`, `failed_rows=Some([...])`, `successful_rows=None`
/// - **All rows succeeded**: Returns `success=true`, `failed_rows=None`, `successful_rows=Some([...])`
///
/// # Examples
///
/// ```no_run
/// use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
/// use arrow::record_batch::RecordBatch;
///
/// # async fn example() -> Result<(), arrow_zerobus_sdk_wrapper::ZerobusError> {
/// # use arrow::array::Int64Array;
/// # use arrow::datatypes::{DataType, Field, Schema};
/// # use std::sync::Arc;
/// # let config = WrapperConfiguration::new("https://workspace.cloud.databricks.com".to_string(), "table".to_string());
/// # let wrapper = ZerobusWrapper::new(config).await?;
/// # let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
/// # let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(Int64Array::from(vec![1, 2, 3]))]).unwrap();
/// let result = wrapper.send_batch(batch.clone()).await?;
///
/// // Check for partial success
/// if result.is_partial_success() {
///     // Extract failed rows for quarantine
///     if let Some(failed_batch) = result.extract_failed_batch(&batch) {
///         // Quarantine failed_batch
///     }
///     
///     // Extract successful rows for writing
///     if let Some(successful_batch) = result.extract_successful_batch(&batch) {
///         // Write successful_batch to main table
///     }
/// }
///
/// // Analyze error patterns
/// let stats = result.get_error_statistics();
/// println!("Success rate: {:.1}%", stats.success_rate * 100.0);
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TransmissionResult {
    /// Whether transmission succeeded
    ///
    /// `true` if ANY rows succeeded, `false` if ALL rows failed or batch-level error occurred.
    pub success: bool,
    /// Error information if transmission failed at batch level
    ///
    /// Batch-level errors occur before per-row processing (e.g., authentication failure,
    /// connection error). If `Some`, indicates no per-row processing occurred.
    pub error: Option<ZerobusError>,
    /// Number of retry attempts made
    pub attempts: u32,
    /// Transmission latency in milliseconds (if successful)
    pub latency_ms: Option<u64>,
    /// Size of transmitted batch in bytes
    pub batch_size_bytes: usize,
    /// Indices of rows that failed, along with their specific errors
    ///
    /// - `None` if all rows succeeded
    /// - `Some(vec![])` if batch-level error only (no per-row processing occurred)
    /// - `Some(vec![(row_idx, error), ...])` for per-row failures
    ///
    /// Each tuple contains:
    /// - `row_idx`: 0-based index of the failed row in the original batch
    /// - `error`: Specific `ZerobusError` describing why the row failed
    pub failed_rows: Option<Vec<(usize, ZerobusError)>>,
    /// Indices of rows that were successfully written
    ///
    /// - `None` if all rows failed
    /// - `Some(vec![])` if no rows succeeded
    /// - `Some(vec![row_idx, ...])` for successful rows
    ///
    /// Each value is a 0-based index of the successful row in the original batch.
    pub successful_rows: Option<Vec<usize>>,
    /// Total number of rows in the batch
    ///
    /// Always equals `successful_count + failed_count`.
    /// For empty batches, this is `0`.
    pub total_rows: usize,
    /// Number of rows that succeeded
    ///
    /// Always equals `successful_rows.len()` if `successful_rows` is `Some`.
    pub successful_count: usize,
    /// Number of rows that failed
    ///
    /// Always equals `failed_rows.len()` if `failed_rows` is `Some`.
    pub failed_count: usize,
}

impl TransmissionResult {
    /// Check if this result represents a partial success (some rows succeeded, some failed)
    ///
    /// Returns `true` if there are both successful and failed rows.
    pub fn is_partial_success(&self) -> bool {
        self.success && self.successful_count > 0 && self.failed_count > 0
    }

    /// Check if there are any failed rows
    ///
    /// Returns `true` if `failed_rows` contains any entries.
    pub fn has_failed_rows(&self) -> bool {
        self.failed_rows
            .as_ref()
            .map(|rows| !rows.is_empty())
            .unwrap_or(false)
    }

    /// Check if there are any successful rows
    ///
    /// Returns `true` if `successful_rows` contains any entries.
    pub fn has_successful_rows(&self) -> bool {
        self.successful_rows
            .as_ref()
            .map(|rows| !rows.is_empty())
            .unwrap_or(false)
    }

    /// Get indices of failed rows
    ///
    /// Returns a vector of row indices that failed, or empty vector if none failed.
    pub fn get_failed_row_indices(&self) -> Vec<usize> {
        self.failed_rows
            .as_ref()
            .map(|rows| rows.iter().map(|(idx, _)| *idx).collect())
            .unwrap_or_default()
    }

    /// Get indices of successful rows
    ///
    /// Returns a vector of row indices that succeeded, or empty vector if none succeeded.
    pub fn get_successful_row_indices(&self) -> Vec<usize> {
        self.successful_rows.clone().unwrap_or_default()
    }

    /// Extract a RecordBatch containing only the failed rows from the original batch
    ///
    /// # Arguments
    ///
    /// * `original_batch` - The original RecordBatch that was sent
    ///
    /// # Returns
    ///
    /// Returns `Some(RecordBatch)` containing only the failed rows, or `None` if there are no failed rows.
    /// Rows are extracted in the order they appear in `failed_rows`.
    pub fn extract_failed_batch(&self, original_batch: &RecordBatch) -> Option<RecordBatch> {
        let failed_indices = self.get_failed_row_indices();
        if failed_indices.is_empty() {
            return None;
        }

        // Extract rows by index
        let mut rows_to_extract = failed_indices;
        rows_to_extract.sort(); // Ensure consistent ordering

        // Use take to extract specific row indices
        // Note: This requires Arrow's take kernel functionality
        // For now, we'll use a simple approach: filter the batch
        let mut arrays = Vec::new();
        for array in original_batch.columns() {
            // Use take to extract rows at specific indices
            let taken = arrow::compute::take(
                array,
                &arrow::array::UInt32Array::from(
                    rows_to_extract
                        .iter()
                        .map(|&idx| idx as u32)
                        .collect::<Vec<_>>(),
                ),
                None,
            )
            .ok()?;
            arrays.push(taken);
        }

        RecordBatch::try_new(original_batch.schema(), arrays).ok()
    }

    /// Extract a RecordBatch containing only the successful rows from the original batch
    ///
    /// # Arguments
    ///
    /// * `original_batch` - The original RecordBatch that was sent
    ///
    /// # Returns
    ///
    /// Returns `Some(RecordBatch)` containing only the successful rows, or `None` if there are no successful rows.
    /// Rows are extracted in the order they appear in `successful_rows`.
    pub fn extract_successful_batch(&self, original_batch: &RecordBatch) -> Option<RecordBatch> {
        let successful_indices = self.get_successful_row_indices();
        if successful_indices.is_empty() {
            return None;
        }

        // Extract rows by index
        let mut rows_to_extract = successful_indices;
        rows_to_extract.sort(); // Ensure consistent ordering

        // Use take to extract specific row indices
        let mut arrays = Vec::new();
        for array in original_batch.columns() {
            // Use take to extract rows at specific indices
            let taken = arrow::compute::take(
                array,
                &arrow::array::UInt32Array::from(
                    rows_to_extract
                        .iter()
                        .map(|&idx| idx as u32)
                        .collect::<Vec<_>>(),
                ),
                None,
            )
            .ok()?;
            arrays.push(taken);
        }

        RecordBatch::try_new(original_batch.schema(), arrays).ok()
    }

    /// Get indices of failed rows filtered by error type
    ///
    /// # Arguments
    ///
    /// * `predicate` - A closure that returns `true` for errors that should be included
    ///
    /// # Returns
    ///
    /// Returns a vector of row indices for failed rows that match the predicate.
    pub fn get_failed_row_indices_by_error_type<F>(&self, predicate: F) -> Vec<usize>
    where
        F: Fn(&ZerobusError) -> bool,
    {
        self.failed_rows
            .as_ref()
            .map(|rows| {
                rows.iter()
                    .filter_map(
                        |(idx, error)| {
                            if predicate(error) {
                                Some(*idx)
                            } else {
                                None
                            }
                        },
                    )
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Group failed rows by error type
    ///
    /// # Returns
    ///
    /// Returns a HashMap where keys are error type names (e.g., "ConversionError")
    /// and values are vectors of row indices that failed with that error type.
    pub fn group_errors_by_type(&self) -> std::collections::HashMap<String, Vec<usize>> {
        let mut grouped: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();

        if let Some(failed_rows) = &self.failed_rows {
            for (row_idx, error) in failed_rows {
                let error_type = match error {
                    ZerobusError::ConfigurationError(_) => "ConfigurationError",
                    ZerobusError::AuthenticationError(_) => "AuthenticationError",
                    ZerobusError::ConnectionError(_) => "ConnectionError",
                    ZerobusError::ConversionError(_) => "ConversionError",
                    ZerobusError::TransmissionError(_) => "TransmissionError",
                    ZerobusError::RetryExhausted(_) => "RetryExhausted",
                    ZerobusError::TokenRefreshError(_) => "TokenRefreshError",
                };
                grouped
                    .entry(error_type.to_string())
                    .or_default()
                    .push(*row_idx);
            }
        }

        grouped
    }

    /// Get error statistics for this transmission result
    ///
    /// # Returns
    ///
    /// Returns an `ErrorStatistics` struct containing comprehensive error analysis
    /// including success/failure rates and error type counts.
    pub fn get_error_statistics(&self) -> ErrorStatistics {
        let success_rate = if self.total_rows > 0 {
            self.successful_count as f64 / self.total_rows as f64
        } else {
            0.0
        };

        let failure_rate = if self.total_rows > 0 {
            self.failed_count as f64 / self.total_rows as f64
        } else {
            0.0
        };

        let mut error_type_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        if let Some(failed_rows) = &self.failed_rows {
            for (_, error) in failed_rows {
                let error_type = match error {
                    ZerobusError::ConfigurationError(_) => "ConfigurationError",
                    ZerobusError::AuthenticationError(_) => "AuthenticationError",
                    ZerobusError::ConnectionError(_) => "ConnectionError",
                    ZerobusError::ConversionError(_) => "ConversionError",
                    ZerobusError::TransmissionError(_) => "TransmissionError",
                    ZerobusError::RetryExhausted(_) => "RetryExhausted",
                    ZerobusError::TokenRefreshError(_) => "TokenRefreshError",
                };
                *error_type_counts.entry(error_type.to_string()).or_insert(0) += 1;
            }
        }

        ErrorStatistics {
            total_rows: self.total_rows,
            successful_count: self.successful_count,
            failed_count: self.failed_count,
            success_rate,
            failure_rate,
            error_type_counts,
        }
    }

    /// Get all error messages from failed rows
    ///
    /// # Returns
    ///
    /// Returns a vector of error message strings for all failed rows.
    pub fn get_error_messages(&self) -> Vec<String> {
        self.failed_rows
            .as_ref()
            .map(|rows| rows.iter().map(|(_, error)| error.to_string()).collect())
            .unwrap_or_default()
    }
}

/// Error statistics for a transmission result
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    /// Total number of rows in the batch
    pub total_rows: usize,
    /// Number of rows that succeeded
    pub successful_count: usize,
    /// Number of rows that failed
    pub failed_count: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
    /// Count of errors by type
    pub error_type_counts: std::collections::HashMap<String, usize>,
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

        // Initialize debug writer if any format is enabled
        // Check new flags first, fall back to legacy flag for backward compatibility
        let any_debug_enabled =
            config.debug_arrow_enabled || config.debug_protobuf_enabled || config.debug_enabled;

        // Info logging to diagnose why debug writer isn't being initialized
        info!(
            "ZerobusWrapper::new: debug_arrow_enabled={}, debug_protobuf_enabled={}, debug_enabled={}, debug_output_dir={:?}",
            config.debug_arrow_enabled, config.debug_protobuf_enabled, config.debug_enabled, config.debug_output_dir
        );

        let debug_writer = if any_debug_enabled {
            if let Some(output_dir) = &config.debug_output_dir {
                use crate::wrapper::debug::DebugWriter;
                use std::time::Duration;

                info!(
                    "Initializing debug writer with output_dir: {}, table_name: {}, arrow_enabled: {}, protobuf_enabled: {}",
                    output_dir.display(),
                    config.table_name,
                    config.debug_arrow_enabled,
                    config.debug_protobuf_enabled
                );
                match DebugWriter::new(
                    output_dir.clone(),
                    config.table_name.clone(),
                    Duration::from_secs(config.debug_flush_interval_secs),
                    config.debug_max_file_size,
                    config.debug_max_files_retained,
                ) {
                    Ok(writer) => {
                        info!(
                            "Debug file output enabled: {} (Arrow: {}, Protobuf: {})",
                            output_dir.display(),
                            config.debug_arrow_enabled,
                            config.debug_protobuf_enabled
                        );
                        Some(Arc::new(writer))
                    }
                    Err(e) => {
                        warn!("Failed to initialize debug writer: {}", e);
                        None
                    }
                }
            } else {
                warn!("Debug flags enabled but debug_output_dir is None - debug files will not be written");
                None
            }
        } else {
            info!("All debug flags are false - debug files will not be written");
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

        // Write Arrow batch to debug file if Arrow debug is enabled
        if self.config.debug_arrow_enabled {
            if let Some(ref debug_writer) = self.debug_writer {
                if let Err(e) = debug_writer.write_arrow(&batch).await {
                    warn!("Failed to write Arrow debug file: {}", e);
                    // Don't fail the operation if debug writing fails
                }
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

        let total_rows = batch.num_rows();

        // Handle empty batch edge case
        if total_rows == 0 {
            return Ok(TransmissionResult {
                success: true, // Empty batch is considered successful
                error: None,
                attempts,
                latency_ms: Some(latency_ms),
                batch_size_bytes,
                failed_rows: None,
                successful_rows: None,
                total_rows: 0,
                successful_count: 0,
                failed_count: 0,
            });
        }

        match result {
            Ok(batch_result) => {
                // Merge conversion and transmission errors
                let mut all_failed_rows = batch_result.failed_rows;
                let successful_rows = batch_result.successful_rows;

                let successful_count = successful_rows.len();
                let failed_count = all_failed_rows.len();

                // Determine overall success: true if ANY rows succeeded
                // Edge case: If all rows failed, success is false
                let overall_success = successful_count > 0;

                // Sort failed rows by index for consistency
                all_failed_rows.sort_by_key(|(idx, _)| *idx);

                // Update failure rate tracking (only counts network/transmission errors)
                crate::wrapper::zerobus::update_failure_rate(
                    &self.config.table_name,
                    total_rows,
                    &all_failed_rows,
                );

                Ok(TransmissionResult {
                    success: overall_success,
                    error: None, // No batch-level error, only per-row errors
                    attempts,
                    latency_ms: Some(latency_ms),
                    batch_size_bytes,
                    failed_rows: if all_failed_rows.is_empty() {
                        None
                    } else {
                        Some(all_failed_rows)
                    },
                    successful_rows: if successful_rows.is_empty() {
                        None
                    } else {
                        Some(successful_rows)
                    },
                    total_rows,
                    successful_count,
                    failed_count,
                })
            }
            Err(e) => {
                error!("Failed to send batch after retries: {}", e);
                // Batch-level error (e.g., authentication, connection before processing)
                // Edge case: Batch-level errors occur before per-row processing
                Ok(TransmissionResult {
                    success: false,
                    error: Some(e),
                    attempts,
                    latency_ms: Some(latency_ms),
                    batch_size_bytes,
                    failed_rows: None, // Batch-level error, no per-row processing occurred
                    successful_rows: None,
                    total_rows,
                    successful_count: 0,
                    failed_count: 0, // Batch-level error, no per-row processing
                })
            }
        }
    }

    /// Internal method to send a batch (without retry wrapper)
    /// Returns per-row transmission information
    async fn send_batch_internal(
        &self,
        batch: RecordBatch,
        descriptor: Option<prost_types::DescriptorProto>,
    ) -> Result<BatchTransmissionResult, ZerobusError> {
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

        // Write descriptor to file once per table (if either Arrow or Protobuf debug is enabled)
        if self.config.debug_arrow_enabled || self.config.debug_protobuf_enabled {
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
        }

        // 3. Convert Arrow RecordBatch to Protobuf bytes (one per row)
        // This now returns ProtobufConversionResult with per-row conversion errors
        let conversion_result =
            crate::wrapper::conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);

        // Track conversion errors (will be merged with transmission errors later)
        let conversion_errors = conversion_result.failed_rows;

        // Write Protobuf bytes to debug file if Protobuf debug is enabled (only successful conversions)
        // Flush after each batch to ensure files are immediately available for debugging
        // CRITICAL: Write protobuf files BEFORE Zerobus write attempts, so we have them even if Zerobus fails
        if self.config.debug_protobuf_enabled {
            if let Some(ref debug_writer) = self.debug_writer {
                info!(
                    "Writing {} protobuf messages to debug file",
                    conversion_result.successful_bytes.len()
                );
                let num_rows = conversion_result.successful_bytes.len();
                for (idx, (_, bytes)) in conversion_result.successful_bytes.iter().enumerate() {
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
                warn!("⚠️  Debug writer is None - protobuf debug files will not be written. Check debug_protobuf_enabled and debug_output_dir config.");
            }
        }

        // Check if writer is disabled - if so, skip all SDK calls and return success
        // Performance: Operations complete in <50ms (excluding file I/O) when writer disabled
        // This enables performance testing of conversion logic without network overhead
        if self.config.zerobus_writer_disabled {
            debug!(
                "Writer disabled mode enabled - skipping Zerobus SDK calls. Debug files written successfully."
            );
            // Return success with conversion results tracked
            // All successfully converted rows are considered successful when writer is disabled
            let successful_indices: Vec<usize> = conversion_result
                .successful_bytes
                .iter()
                .map(|(idx, _)| *idx)
                .collect();
            return Ok(BatchTransmissionResult {
                successful_rows: successful_indices,
                failed_rows: conversion_errors,
            });
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
        // STEP 5: Check backoff conditions BEFORE attempting any writes
        // ========================================================================
        // CRITICAL: Check backoff BEFORE attempting writes, even if stream exists.
        // This prevents writes during backoff period even if stream was created before
        // backoff started. We check for:
        // 1. Error 6006 backoff (server-initiated, pipeline blocked)
        // 2. High failure rate backoff (client-initiated, >1% failure rate)
        //
        // Edge case: Backoff can start during batch processing, so we check again
        // before each record in the loop below.
        {
            use crate::wrapper::zerobus::{check_error_6006_backoff, check_failure_rate_backoff};
            check_error_6006_backoff(&self.config.table_name).await?;
            check_failure_rate_backoff(&self.config.table_name).await?;
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

        // Track per-row transmission results across retries
        // These will be assigned from attempt_* variables after processing completes
        let mut transmission_errors: Vec<(usize, ZerobusError)> = Vec::new();
        let mut successful_indices: Vec<usize> = Vec::new();

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

            // Try to send all successfully converted rows
            // Reset tracking for this retry attempt (but preserve across retries for final result)
            let mut attempt_transmission_errors: Vec<(usize, ZerobusError)> = Vec::new();
            let mut attempt_successful_indices: Vec<usize> = Vec::new();
            let mut all_succeeded = true;
            let mut failed_at_idx = 0;

            // Batch futures for better throughput: collect futures and await in batches
            // This allows the SDK to queue multiple records before flushing, improving performance
            const BATCH_SIZE: usize = 1000; // Flush every 1000 records
            const BATCH_SIZE_BYTES: usize = 10 * 1024 * 1024; // Or every 10MB
                                                              // Store futures with their row indices - using a type-erased future
            type IngestFuture = std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = Result<i64, databricks_zerobus_ingest_sdk::ZerobusError>,
                        > + Send,
                >,
            >;
            let mut pending_futures: Vec<(usize, IngestFuture)> = Vec::new();
            let mut total_bytes_buffered = 0usize;
            let mut should_break_outer = false; // Track if we need to break outer retry loop

            // Process only successfully converted rows
            for (original_row_idx, bytes) in conversion_result.successful_bytes.iter() {
                let idx = *original_row_idx;
                // ========================================================================
                // STEP 6a: Check backoff before each record
                // ========================================================================
                // Edge case: Backoff can start during batch processing (e.g., another thread
                // encountered error 6006 or high failure rate). We check before each record to prevent writes
                // during backoff period.
                {
                    use crate::wrapper::zerobus::{
                        check_error_6006_backoff, check_failure_rate_backoff,
                    };
                    if let Err(_backoff_err) =
                        check_error_6006_backoff(&self.config.table_name).await
                    {
                        // Backoff error: track per-row and break (backoff is batch-level concern)
                        // Clear stream so it gets recreated after backoff
                        let mut stream_guard = self.stream.lock().await;
                        *stream_guard = None;
                        drop(stream_guard);
                        // Backoff affects remaining rows, but we've processed up to idx
                        // Mark remaining rows as affected by backoff
                        for remaining_idx in idx..conversion_result.successful_bytes.len() {
                            if let Some((orig_idx, _)) =
                                conversion_result.successful_bytes.get(remaining_idx)
                            {
                                attempt_transmission_errors.push((
                                    *orig_idx,
                                    ZerobusError::ConnectionError(
                                        "Backoff period active - row processing stopped"
                                            .to_string(),
                                    ),
                                ));
                            }
                        }
                        all_succeeded = false;
                        failed_at_idx = idx;
                        break;
                    }
                    // Also check failure rate backoff
                    if let Err(_backoff_err) =
                        check_failure_rate_backoff(&self.config.table_name).await
                    {
                        // Backoff error: track per-row and break (backoff is batch-level concern)
                        // Clear stream so it gets recreated after backoff
                        let mut stream_guard = self.stream.lock().await;
                        *stream_guard = None;
                        drop(stream_guard);
                        // Backoff affects remaining rows, but we've processed up to idx
                        // Mark remaining rows as affected by backoff
                        for remaining_idx in idx..conversion_result.successful_bytes.len() {
                            if let Some((orig_idx, _)) =
                                conversion_result.successful_bytes.get(remaining_idx)
                            {
                                attempt_transmission_errors.push((
                                    *orig_idx,
                                    ZerobusError::ConnectionError(
                                        "High failure rate backoff active - row processing stopped"
                                            .to_string(),
                                    ),
                                ));
                            }
                        }
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
                // STEP 6c: Send bytes to Zerobus stream (batched for performance)
                // ========================================================================
                // The Zerobus SDK's ingest_record returns a Future that resolves when acknowledged.
                // We collect futures and await them in batches for better throughput.
                //
                // Error handling:
                // - Stream closed errors: Clear stream, mark failure, break loop to retry
                // - Other errors: Track per-row and continue
                // - First record failures: Log detailed diagnostics for schema issues
                match stream.ingest_record(bytes.clone()).await {
                    Ok(ingest_future) => {
                        // Release lock before collecting future to avoid blocking
                        drop(stream_guard);

                        // Collect future for batch processing
                        // Box the future to store in Vec (type erasure for different future types)
                        pending_futures.push((idx, Box::pin(ingest_future)));
                        total_bytes_buffered += bytes.len();

                        // Periodically flush and await futures to manage memory and ensure progress
                        if pending_futures.len() >= BATCH_SIZE
                            || total_bytes_buffered >= BATCH_SIZE_BYTES
                        {
                            // Flush stream to send buffered records
                            {
                                let mut stream_guard = self.stream.lock().await;
                                if let Some(ref mut stream) = *stream_guard {
                                    if let Err(e) = stream.flush().await {
                                        error!(
                                            "Failed to flush Zerobus stream during batch: {}",
                                            e
                                        );
                                        // Mark all pending futures as failed
                                        for (pending_idx, _) in pending_futures.drain(..) {
                                            attempt_transmission_errors.push((
                                                pending_idx,
                                                ZerobusError::ConnectionError(format!(
                                                    "Flush failed during batch processing: {}",
                                                    e
                                                )),
                                            ));
                                        }
                                        all_succeeded = false;
                                        failed_at_idx = idx;
                                        break;
                                    }
                                }
                            }

                            // Await all pending futures and track results
                            for (pending_idx, mut future) in pending_futures.drain(..) {
                                match future.as_mut().await {
                                    Ok(_ack_id) => {
                                        debug!(
                                            "✅ Successfully sent record to Zerobus stream (row {}, ack_id={})",
                                            pending_idx, _ack_id
                                        );
                                        attempt_successful_indices.push(pending_idx);
                                    }
                                    Err(e) => {
                                        let err_msg = format!("{}", e);
                                        // Check if stream is closed
                                        if err_msg.contains("Stream is closed")
                                            || err_msg.contains("Stream closed")
                                        {
                                            let is_first = pending_idx == 0;
                                            error!(
                                                "Stream closed: row={}, first_record={}, error={}",
                                                pending_idx, is_first, err_msg
                                            );
                                            if is_first {
                                                error!("Diagnostics: Stream closed during batch processing");
                                                error!("Possible causes:");
                                                error!("  1. Schema mismatch between descriptor and table");
                                                error!("  2. Validation error");
                                                error!("  3. Server-side issue");
                                            }
                                            // Clear stream and break to retry
                                            let mut stream_guard = self.stream.lock().await;
                                            *stream_guard = None;
                                            drop(stream_guard);
                                            attempt_transmission_errors.push((
                                                pending_idx,
                                                ZerobusError::ConnectionError(format!(
                                                    "Stream closed: row={}, error={}",
                                                    pending_idx, err_msg
                                                )),
                                            ));
                                            all_succeeded = false;
                                            failed_at_idx = pending_idx;
                                            break;
                                        } else {
                                            // Non-stream-closure errors
                                            attempt_transmission_errors.push((
                                                pending_idx,
                                                ZerobusError::TransmissionError(format!(
                                                    "Record ingestion failed: row={}, error={}",
                                                    pending_idx, e
                                                )),
                                            ));
                                            all_succeeded = false;
                                        }
                                    }
                                }
                            }
                            total_bytes_buffered = 0;

                            // If we broke due to stream closure, mark for outer loop break
                            // But continue to process remaining pending futures below
                            if !all_succeeded && failed_at_idx > 0 {
                                should_break_outer = true;
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
                            // Stream closure error: track per-row and continue
                            // Clear stream so it gets recreated on next iteration
                            *stream_guard = None;
                            drop(stream_guard);
                            let stream_error = ZerobusError::ConnectionError(format!(
                                "Stream closed: row={}, error={}",
                                idx, err_msg
                            ));
                            attempt_transmission_errors.push((idx, stream_error));
                            all_succeeded = false;
                            failed_at_idx = idx;
                            // Mark for outer loop break, but continue to process pending futures
                            should_break_outer = true;
                            break;
                        } else {
                            // Non-stream-closure errors: track per-row and continue
                            let transmission_error = ZerobusError::ConnectionError(format!(
                                "Record creation failed: row={}, error={}",
                                idx, e
                            ));
                            attempt_transmission_errors.push((idx, transmission_error));
                            all_succeeded = false;
                            // Continue processing remaining rows instead of returning immediately
                        }
                    }
                }
            }

            // CRITICAL: Flush and await any remaining pending futures before proceeding
            // This ensures all queued records are sent and acknowledged, even if we broke early
            if !pending_futures.is_empty() {
                // Always flush remaining records before awaiting acknowledgments
                // This ensures records are sent even if we broke early due to errors
                {
                    let mut stream_guard = self.stream.lock().await;
                    if let Some(ref mut stream) = *stream_guard {
                        // Attempt to flush - if stream is closed, this will fail but we still want to await futures
                        match stream.flush().await {
                            Ok(_) => {
                                debug!(
                                    "✅ Flushed Zerobus stream for {} remaining pending futures",
                                    pending_futures.len()
                                );
                            }
                            Err(e) => {
                                warn!("Failed to flush Zerobus stream for remaining records (stream may be closed): {}", e);
                                // Don't mark futures as failed yet - await them to get actual acknowledgment status
                                // The stream might be closed, but some records may have been sent before closure
                            }
                        }
                    } else {
                        warn!("Stream is None when trying to flush remaining records - records may be lost");
                        // Mark all pending futures as failed since we can't flush
                        for (pending_idx, _) in pending_futures.drain(..) {
                            attempt_transmission_errors.push((
                                pending_idx,
                                ZerobusError::ConnectionError(
                                    "Stream was closed before flushing remaining records"
                                        .to_string(),
                                ),
                            ));
                        }
                        all_succeeded = false;
                    }
                }

                // CRITICAL: Always await all pending futures to get acknowledgment status
                // Even if stream is closed, we need to know which records succeeded/failed
                for (pending_idx, mut future) in pending_futures.drain(..) {
                    match future.as_mut().await {
                        Ok(_ack_id) => {
                            debug!(
                                "✅ Successfully acknowledged record (row {}, ack_id={})",
                                pending_idx, _ack_id
                            );
                            attempt_successful_indices.push(pending_idx);
                        }
                        Err(e) => {
                            let err_msg = format!("{}", e);
                            if err_msg.contains("Stream is closed")
                                || err_msg.contains("Stream closed")
                            {
                                // Stream was closed - clear it and mark as failed
                                let mut stream_guard = self.stream.lock().await;
                                *stream_guard = None;
                                drop(stream_guard);
                                attempt_transmission_errors.push((
                                    pending_idx,
                                    ZerobusError::ConnectionError(format!(
                                        "Stream closed before acknowledgment: row={}, error={}",
                                        pending_idx, err_msg
                                    )),
                                ));
                                all_succeeded = false;
                            } else {
                                // Other errors (network, timeout, etc.)
                                attempt_transmission_errors.push((
                                    pending_idx,
                                    ZerobusError::TransmissionError(format!(
                                        "Record acknowledgment failed: row={}, error={}",
                                        pending_idx, e
                                    )),
                                ));
                                all_succeeded = false;
                            }
                        }
                    }
                }
            }

            // If we broke early due to stream closure, exit the retry loop
            if should_break_outer {
                break;
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
                // All rows sent successfully - flush stream to ensure records are transmitted
                // CRITICAL: The SDK buffers records internally and requires flush() to send them
                {
                    let mut stream_guard = self.stream.lock().await;
                    if let Some(ref mut stream) = *stream_guard {
                        if let Err(e) = stream.flush().await {
                            error!("Failed to flush Zerobus stream after batch: {}", e);
                            // Don't fail the entire batch if flush fails - records may still be in transit
                            // But log the error for monitoring
                        } else {
                            debug!(
                                "✅ Flushed Zerobus stream after sending {} records",
                                attempt_successful_indices.len()
                            );
                        }
                    }
                }
                // Update final results with this attempt's results
                successful_indices = attempt_successful_indices;
                transmission_errors = attempt_transmission_errors;
                break;
            } else {
                // Some rows failed due to stream closure - retry with stream recreation
                retry_count += 1;
                if retry_count > MAX_STREAM_RECREATE_ATTEMPTS {
                    // Exhausted retry attempts - use what we have from this attempt
                    let mut final_transmission_errors = attempt_transmission_errors;
                    let final_successful_indices = attempt_successful_indices;
                    // Mark remaining rows as failed due to stream closure
                    for (idx, _) in conversion_result.successful_bytes.iter() {
                        if !final_successful_indices.contains(idx)
                            && !final_transmission_errors.iter().any(|(i, _)| i == idx)
                        {
                            final_transmission_errors.push((*idx, ZerobusError::ConnectionError(format!(
                                "Stream recreation exhausted: row={}, possible_causes='schema_mismatch,validation_error,server_issue'",
                                idx
                            ))));
                        }
                    }
                    successful_indices = final_successful_indices;
                    transmission_errors = final_transmission_errors;
                    break;
                }
                warn!(
                    "Stream recreation retry: attempt={}/{}, failed_at_row={}",
                    retry_count, MAX_STREAM_RECREATE_ATTEMPTS, failed_at_idx
                );
                // Small delay before retry to avoid tight retry loops
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                // Reset attempt tracking for retry - will retry all remaining rows
                attempt_successful_indices.clear();
                attempt_transmission_errors.clear();
                // Note: all_succeeded will be set to true at start of next loop iteration
            }
        }

        // Merge conversion errors with transmission errors
        let mut all_failed_rows = conversion_errors;
        all_failed_rows.extend(transmission_errors);
        Ok(BatchTransmissionResult {
            successful_rows: successful_indices,
            failed_rows: all_failed_rows,
        })
    }

    /// Flush any pending operations and ensure data is transmitted
    ///
    /// # Errors
    ///
    /// Returns error if flush operation fails.
    pub async fn flush(&self) -> Result<(), ZerobusError> {
        // CRITICAL: Flush Zerobus stream to ensure buffered records are sent
        // The SDK buffers records internally and requires flush() to transmit them
        {
            let mut stream_guard = self.stream.lock().await;
            if let Some(ref mut stream) = *stream_guard {
                stream.flush().await.map_err(|e| {
                    ZerobusError::ConnectionError(format!("Failed to flush Zerobus stream: {}", e))
                })?;
                debug!("✅ Flushed Zerobus stream");
            }
        }

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

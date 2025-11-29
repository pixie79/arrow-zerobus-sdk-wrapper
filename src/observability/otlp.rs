//! OpenTelemetry integration via otlp-rust-service
//!
//! This module uses the otlp-rust-service SDK for OpenTelemetry functionality.
//! Metrics and traces are recorded via tracing, which the SDK infrastructure
//! picks up and converts to OpenTelemetry format for export.
//!
//! The SDK handles all OpenTelemetry data structure creation internally,
//! eliminating the need for manual ResourceMetrics or SpanData construction.

use crate::config::OtlpSdkConfig;
use crate::error::ZerobusError;

#[cfg(feature = "observability")]
use std::sync::Arc;

#[cfg(feature = "observability")]
use otlp_arrow_library::{Config as OtlpLibraryConfig, OtlpLibrary};

/// Observability manager for collecting metrics and traces
///
/// Wraps the otlp-rust-service library to provide OpenTelemetry
/// metrics and trace collection for the wrapper.
#[derive(Clone)]
pub struct ObservabilityManager {
    #[cfg(feature = "observability")]
    library: Option<Arc<OtlpLibrary>>,
    #[cfg(not(feature = "observability"))]
    _phantom: std::marker::PhantomData<()>,
}

impl ObservabilityManager {
    /// Create observability manager asynchronously
    ///
    /// This method properly initializes the OtlpLibrary asynchronously.
    ///
    /// # Arguments
    ///
    /// * `config` - Optional OTLP SDK configuration. If None, observability is disabled.
    ///
    /// # Returns
    ///
    /// Returns `Some(ObservabilityManager)` if observability is enabled and
    /// initialization succeeds, or `None` if disabled or initialization fails.
    pub async fn new_async(config: Option<OtlpSdkConfig>) -> Option<Self> {
        let _config = match config {
            Some(c) => c,
            None => return None,
        };

        #[cfg(feature = "observability")]
        {
            use otlp_arrow_library::ConfigBuilder;

            // Build SDK config directly from OtlpSdkConfig
            let mut builder = ConfigBuilder::default();

            // Set output directory if provided
            if let Some(output_dir) = &_config.output_dir {
                builder = builder.output_dir(output_dir.clone());
            }

            // Set write interval
            builder = builder.write_interval_secs(_config.write_interval_secs);

            // Configure tracing log level
            let log_level = _config.log_level.to_lowercase();
            std::env::set_var(
                "RUST_LOG",
                format!("arrow_zerobus_sdk_wrapper={}", log_level),
            );

            // Build config, using defaults if build fails
            let library_config = builder.build().unwrap_or_else(|_| {
                tracing::warn!("Failed to build SDK config, using defaults");
                OtlpLibraryConfig::default()
            });

            match OtlpLibrary::new(library_config).await {
                Ok(library) => Some(Self {
                    library: Some(Arc::new(library)),
                }),
                Err(e) => {
                    tracing::warn!("Failed to initialize OtlpLibrary: {}", e);
                    None
                }
            }
        }

        #[cfg(not(feature = "observability"))]
        {
            None
        }
    }

    /// Record a batch transmission metric
    ///
    /// Uses tracing to record metrics, which are picked up by the otlp-rust-service SDK
    /// infrastructure and converted to OpenTelemetry metrics.
    ///
    /// # Arguments
    ///
    /// * `batch_size_bytes` - Size of the batch in bytes
    /// * `success` - Whether transmission succeeded
    /// * `latency_ms` - Transmission latency in milliseconds
    pub async fn record_batch_sent(&self, batch_size_bytes: usize, success: bool, latency_ms: u64) {
        #[cfg(feature = "observability")]
        {
            if self.library.is_some() {
                // Record metrics via tracing with structured fields
                // The otlp-rust-service SDK infrastructure picks up these tracing events
                // and converts them to OpenTelemetry metrics
                tracing::info!(
                    metric.name = "zerobus.batch.size_bytes",
                    metric.value = batch_size_bytes,
                    metric.unit = "bytes",
                    batch_size_bytes = batch_size_bytes,
                    success = success,
                    latency_ms = latency_ms,
                    "zerobus.batch.metrics"
                );

                tracing::info!(
                    metric.name = "zerobus.batch.success",
                    metric.value = if success { 1i64 } else { 0i64 },
                    success = success,
                    "zerobus.batch.metrics"
                );

                tracing::info!(
                    metric.name = "zerobus.batch.latency_ms",
                    metric.value = latency_ms,
                    metric.unit = "ms",
                    latency_ms = latency_ms,
                    "zerobus.batch.metrics"
                );
            }
        }

        #[cfg(not(feature = "observability"))]
        {
            let _ = (batch_size_bytes, success, latency_ms);
        }
    }

    /// Start a span for batch transmission operation
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the target table
    ///
    /// # Returns
    ///
    /// Returns a span guard that ends the span when dropped
    pub fn start_send_batch_span(&self, table_name: &str) -> ObservabilitySpan {
        let start_time = std::time::SystemTime::now();
        #[cfg(feature = "observability")]
        {
            // Create a span for the operation
            // The span will be exported when dropped with correct timing
            ObservabilitySpan {
                _table_name: table_name.to_string(),
                start_time,
                library: self.library.clone(),
            }
        }

        #[cfg(not(feature = "observability"))]
        {
            let _ = table_name;
            ObservabilitySpan {
                _table_name: String::new(),
                start_time,
            }
        }
    }

    /// Flush pending observability data
    pub async fn flush(&self) -> Result<(), ZerobusError> {
        #[cfg(feature = "observability")]
        {
            if let Some(library) = &self.library {
                library.flush().await.map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to flush observability data: {}",
                        e
                    ))
                })?;
            }
        }
        Ok(())
    }

    /// Shutdown the observability manager
    pub async fn shutdown(&self) -> Result<(), ZerobusError> {
        #[cfg(feature = "observability")]
        {
            if let Some(library) = &self.library {
                library.shutdown().await.map_err(|e| {
                    ZerobusError::ConfigurationError(format!(
                        "Failed to shutdown observability: {}",
                        e
                    ))
                })?;
            }
        }
        Ok(())
    }
}

/// Span guard for observability operations
///
/// When dropped, automatically ends the span with the correct end time.
pub struct ObservabilitySpan {
    _table_name: String,
    #[allow(dead_code)] // Used in Drop impl
    start_time: std::time::SystemTime,
    #[cfg(feature = "observability")]
    library: Option<Arc<OtlpLibrary>>,
}

impl Drop for ObservabilitySpan {
    fn drop(&mut self) {
        #[cfg(feature = "observability")]
        {
            if self.library.is_some() {
                let end_time = std::time::SystemTime::now();
                let duration = end_time
                    .duration_since(self.start_time)
                    .unwrap_or_default()
                    .as_millis() as u64;

                // Record span completion via tracing
                // The otlp-rust-service SDK infrastructure picks up these tracing events
                // and converts them to OpenTelemetry traces
                tracing::info!(
                    span.name = "zerobus.send_batch",
                    span.table_name = %self._table_name,
                    span.duration_ms = duration,
                    "zerobus.send_batch.completed"
                );
            }
        }
    }
}

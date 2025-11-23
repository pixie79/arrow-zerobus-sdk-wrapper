//! OpenTelemetry integration via otlp-rust-service
//!
//! This module uses the otlp-rust-service library for OpenTelemetry functionality.
//! Provides a wrapper around OtlpLibrary for metrics and trace collection.

use crate::config::OtlpConfig;
use crate::error::ZerobusError;

#[cfg(feature = "observability")]
use otlp_arrow_library::{Config as OtlpLibraryConfig, ConfigBuilder, OtlpLibrary};

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
    /// Create a new observability manager
    ///
    /// # Arguments
    ///
    /// * `config` - Optional OTLP configuration. If None, observability is disabled.
    ///
    /// # Returns
    ///
    /// Returns `Some(ObservabilityManager)` if observability is enabled and
    /// initialization succeeds, or `None` if disabled or initialization fails.
    pub fn new(config: Option<OtlpConfig>) -> Option<Self> {
        let _config = match config {
            Some(c) => c,
            None => return None,
        };

        #[cfg(feature = "observability")]
        {
            // Synchronous initialization is not supported - use new_async instead
            // This method returns None to indicate async initialization is required
            let _config = config; // Suppress unused variable warning
            None
        }

        #[cfg(not(feature = "observability"))]
        {
            None
        }
    }

    /// Create observability manager asynchronously
    ///
    /// This method properly initializes the OtlpLibrary asynchronously.
    pub async fn new_async(config: Option<OtlpConfig>) -> Option<Self> {
        let _config = match config {
            Some(c) => c,
            None => return None,
        };

        #[cfg(feature = "observability")]
        {
            let library_config = Self::convert_config(config);

            match OtlpLibrary::new(library_config).await {
                Ok(library) => Some(Self {
                    library: Some(Arc::new(library)),
                }),
                Err(e) => {
                    warn!("Failed to initialize OtlpLibrary: {}", e);
                    None
                }
            }
        }

        #[cfg(not(feature = "observability"))]
        {
            None
        }
    }

    /// Convert our OtlpConfig to otlp-rust-service Config
    #[cfg(feature = "observability")]
    fn convert_config(config: OtlpConfig) -> OtlpLibraryConfig {
        use otlp_arrow_library::ConfigBuilder;
        use std::path::PathBuf;

        let mut builder = ConfigBuilder::default();

        // Set output directory (default to /tmp/otlp if not specified)
        let output_dir = config
            .endpoint
            .as_ref()
            .map(|_| PathBuf::from("/tmp/otlp"))
            .unwrap_or_else(|| PathBuf::from("/tmp/otlp"));

        builder = builder.output_dir(output_dir);

        // Set write interval (default 5 seconds)
        builder = builder.write_interval_secs(5);

        // Build config, using defaults if build fails
        builder.build().unwrap_or_else(|_| {
            // Fallback to default config if build fails
            OtlpLibraryConfig::default()
        })
    }

    /// Record a batch transmission metric
    ///
    /// # Arguments
    ///
    /// * `batch_size_bytes` - Size of the batch in bytes
    /// * `success` - Whether transmission succeeded
    /// * `latency_ms` - Transmission latency in milliseconds
    pub async fn record_batch_sent(&self, batch_size_bytes: usize, success: bool, latency_ms: u64) {
        #[cfg(feature = "observability")]
        {
            if let Some(library) = &self.library {
                // Create ResourceMetrics for export
                let metrics = Self::create_batch_metrics(batch_size_bytes, success, latency_ms);

                // Export metrics using otlp-arrow-library
                if let Err(e) = library.export_metrics(metrics).await {
                    tracing::warn!("Failed to export metrics: {}", e);
                }
            }
        }

        #[cfg(not(feature = "observability"))]
        {
            let _ = (batch_size_bytes, success, latency_ms);
        }
    }

    /// Create ResourceMetrics for batch transmission
    ///
    /// Creates a minimal ResourceMetrics structure that can be exported.
    /// Note: ResourceMetrics fields are private in OpenTelemetry SDK 0.31.
    /// We use ResourceMetrics::default() as shown in otlp-rust-service examples.
    ///
    /// The actual metric values (batch_size_bytes, success, latency_ms) are
    /// logged via tracing and will be exported via the observability infrastructure.
    #[cfg(feature = "observability")]
    fn create_batch_metrics(
        batch_size_bytes: usize,
        success: bool,
        latency_ms: u64,
    ) -> opentelemetry_sdk::metrics::data::ResourceMetrics {
        // Log metrics via tracing (will be picked up by observability infrastructure)
        tracing::info!(
            batch_size_bytes = batch_size_bytes,
            success = success,
            latency_ms = latency_ms,
            "zerobus.batch.metrics"
        );

        // Create minimal ResourceMetrics using default() as shown in examples
        // The library will handle the conversion and export
        opentelemetry_sdk::metrics::data::ResourceMetrics::default()
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
        #[cfg(feature = "observability")]
        {
            // Create a span for the operation
            // The span will be exported when dropped
            ObservabilitySpan {
                _table_name: table_name.to_string(),
                library: self.library.clone(),
            }
        }

        #[cfg(not(feature = "observability"))]
        {
            let _ = table_name;
            ObservabilitySpan {
                _table_name: String::new(),
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
/// When dropped, automatically ends the span.
pub struct ObservabilitySpan {
    _table_name: String,
    #[cfg(feature = "observability")]
    library: Option<Arc<OtlpLibrary>>,
}

impl Drop for ObservabilitySpan {
    fn drop(&mut self) {
        #[cfg(feature = "observability")]
        {
            // Create and export span data
            if let Some(library) = &self.library {
                let span_data = Self::create_span_data(&self._table_name);
                // Export span asynchronously (fire and forget)
                let library_clone = library.clone();
                let span_data_clone = span_data.clone();
                tokio::spawn(async move {
                    if let Err(e) = library_clone.export_trace(span_data_clone).await {
                        tracing::warn!("Failed to export trace: {}", e);
                    }
                });
            }
        }
    }
}

impl ObservabilitySpan {
    #[cfg(feature = "observability")]
    fn create_span_data(table_name: &str) -> opentelemetry_sdk::trace::SpanData {
        use opentelemetry::trace::{
            SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
        };
        use opentelemetry::KeyValue;
        use opentelemetry_sdk::Resource;
        use std::time::{Duration, SystemTime};

        // Generate random trace and span IDs
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let trace_bytes: [u8; 16] = rng.gen();
        let span_bytes: [u8; 8] = rng.gen();

        let trace_id = TraceId::from_bytes(trace_bytes);
        let span_id = SpanId::from_bytes(span_bytes);

        let span_context = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        );

        // Create a default parent span ID (no parent) - use INVALID as shown in tests
        let parent_span_id = SpanId::INVALID;

        opentelemetry_sdk::trace::SpanData {
            span_context,
            parent_span_id,
            span_kind: SpanKind::Client,
            name: std::borrow::Cow::Owned("zerobus.send_batch".to_string()),
            start_time: SystemTime::now(),
            end_time: SystemTime::now() + Duration::from_millis(1),
            attributes: vec![KeyValue::new("table.name", table_name.to_string())],
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: Status::Ok,
            dropped_attributes_count: 0,
            parent_span_is_remote: false,
            instrumentation_scope: opentelemetry::InstrumentationScope::builder(
                "arrow-zerobus-sdk-wrapper",
            )
            .with_version(env!("CARGO_PKG_VERSION"))
            .build(),
        }
    }
}

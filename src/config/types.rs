//! Configuration types for Zerobus SDK Wrapper
//!
//! This module defines the configuration structures and validation logic.

use crate::error::ZerobusError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// OpenTelemetry configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OtlpConfig {
    /// OTLP endpoint URL (optional, uses default if not provided)
    pub endpoint: Option<String>,
    /// Log level filter for tracing (e.g., "info", "debug", "warn", "error")
    /// Controls which log events are exported via tracing
    /// Default: "info"
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Additional OTLP configuration options
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// OpenTelemetry SDK configuration
///
/// This configuration structure aligns with the otlp-rust-service SDK requirements.
/// It replaces `OtlpConfig` as a breaking change to simplify configuration and
/// directly map to SDK ConfigBuilder fields.
///
/// # Migration from OtlpConfig
///
/// The old `OtlpConfig` structure had:
/// - `endpoint: Option<String>`
/// - `log_level: String`
/// - `extra: HashMap<String, Value>`
///
/// The new `OtlpSdkConfig` structure has:
/// - `endpoint: Option<String>` - OTLP endpoint URL for remote export
/// - `output_dir: Option<PathBuf>` - Output directory for file-based export
/// - `write_interval_secs: u64` - Write interval in seconds (default: 5)
/// - `log_level: String` - Log level for tracing (default: "info")
///
/// The `extra` field has been removed as it's no longer needed with direct SDK config mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpSdkConfig {
    /// OTLP endpoint URL for remote export (optional)
    pub endpoint: Option<String>,
    /// Output directory for file-based export (optional)
    pub output_dir: Option<PathBuf>,
    /// Write interval in seconds for file-based export (default: 5)
    #[serde(default = "default_write_interval")]
    pub write_interval_secs: u64,
    /// Log level for tracing (default: "info")
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_write_interval() -> u64 {
    5
}

/// Complete configuration for initializing the wrapper
///
/// Represents all configuration needed to initialize a ZerobusWrapper instance,
/// including connection details, observability settings, debug file settings,
/// and retry configuration.
#[derive(Debug, Clone)]
pub struct WrapperConfiguration {
    /// Zerobus endpoint URL (required)
    pub zerobus_endpoint: String,
    /// Unity Catalog URL for authentication (required for SDK)
    pub unity_catalog_url: Option<String>,
    /// OAuth2 client ID (required for SDK)
    pub client_id: Option<String>,
    /// OAuth2 client secret (required for SDK)
    pub client_secret: Option<String>,
    /// Target table name in Zerobus (required)
    pub table_name: String,
    /// Enable/disable OpenTelemetry observability (default: false)
    pub observability_enabled: bool,
    /// OpenTelemetry configuration (optional)
    pub observability_config: Option<OtlpSdkConfig>,
    /// Enable/disable debug file output (default: false)
    pub debug_enabled: bool,
    /// Output directory for debug files (required if debug_enabled)
    pub debug_output_dir: Option<PathBuf>,
    /// Debug file flush interval in seconds (default: 5)
    pub debug_flush_interval_secs: u64,
    /// Maximum debug file size in bytes before rotation (optional)
    pub debug_max_file_size: Option<u64>,
    /// Maximum retry attempts for transient failures (default: 5)
    pub retry_max_attempts: u32,
    /// Base delay in milliseconds for exponential backoff (default: 100)
    pub retry_base_delay_ms: u64,
    /// Maximum delay in milliseconds for exponential backoff (default: 30000)
    pub retry_max_delay_ms: u64,
}

impl WrapperConfiguration {
    /// Create a new configuration with defaults
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Zerobus endpoint URL
    /// * `table_name` - Target table name
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arrow_zerobus_sdk_wrapper::WrapperConfiguration;
    ///
    /// let config = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// );
    /// ```
    pub fn new(endpoint: String, table_name: String) -> Self {
        Self {
            zerobus_endpoint: endpoint,
            table_name,
            unity_catalog_url: None,
            client_id: None,
            client_secret: None,
            observability_enabled: false,
            observability_config: None,
            debug_enabled: false,
            debug_output_dir: None,
            debug_flush_interval_secs: 5,
            debug_max_file_size: None,
            retry_max_attempts: 5,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 30000,
        }
    }

    /// Set OAuth2 credentials
    ///
    /// # Arguments
    ///
    /// * `client_id` - OAuth2 client ID
    /// * `client_secret` - OAuth2 client secret
    pub fn with_credentials(mut self, client_id: String, client_secret: String) -> Self {
        self.client_id = Some(client_id);
        self.client_secret = Some(client_secret);
        self
    }

    /// Set Unity Catalog URL
    ///
    /// # Arguments
    ///
    /// * `url` - Unity Catalog URL
    pub fn with_unity_catalog(mut self, url: String) -> Self {
        self.unity_catalog_url = Some(url);
        self
    }

    /// Set OpenTelemetry observability configuration
    ///
    /// # Arguments
    ///
    /// * `config` - OpenTelemetry SDK configuration
    pub fn with_observability(mut self, config: OtlpSdkConfig) -> Self {
        self.observability_enabled = true;
        self.observability_config = Some(config);
        self
    }

    /// Set debug output configuration
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Output directory for debug files
    pub fn with_debug_output(mut self, output_dir: PathBuf) -> Self {
        self.debug_enabled = true;
        self.debug_output_dir = Some(output_dir);
        self
    }

    /// Set debug flush interval
    ///
    /// # Arguments
    ///
    /// * `interval_secs` - Flush interval in seconds
    pub fn with_debug_flush_interval_secs(mut self, interval_secs: u64) -> Self {
        self.debug_flush_interval_secs = interval_secs;
        self
    }

    /// Set debug max file size
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum file size in bytes before rotation
    pub fn with_debug_max_file_size(mut self, max_size: Option<u64>) -> Self {
        self.debug_max_file_size = max_size;
        self
    }

    /// Set retry configuration
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Maximum retry attempts
    /// * `base_delay_ms` - Base delay in milliseconds for exponential backoff
    /// * `max_delay_ms` - Maximum delay in milliseconds
    pub fn with_retry_config(
        mut self,
        max_attempts: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
    ) -> Self {
        self.retry_max_attempts = max_attempts;
        self.retry_base_delay_ms = base_delay_ms;
        self.retry_max_delay_ms = max_delay_ms;
        self
    }

    /// Validate configuration
    ///
    /// Checks that all required fields are present and valid.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if configuration is valid, or `Err(ZerobusError)` if invalid.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if:
    /// - `zerobus_endpoint` is not a valid URL starting with `https://` or `http://`
    /// - `debug_enabled` is true but `debug_output_dir` is not provided
    /// - `retry_max_attempts` is 0
    /// - `debug_flush_interval_secs` is 0
    pub fn validate(&self) -> Result<(), ZerobusError> {
        // Validate endpoint URL
        if !self.zerobus_endpoint.starts_with("https://")
            && !self.zerobus_endpoint.starts_with("http://")
        {
            return Err(ZerobusError::ConfigurationError(format!(
                "zerobus_endpoint must start with 'https://' or 'http://', got: '{}'",
                self.zerobus_endpoint
            )));
        }

        // Validate debug configuration
        if self.debug_enabled && self.debug_output_dir.is_none() {
            return Err(ZerobusError::ConfigurationError(
                "debug_output_dir is required when debug_enabled is true".to_string(),
            ));
        }

        // Validate retry configuration
        if self.retry_max_attempts == 0 {
            return Err(ZerobusError::ConfigurationError(
                "retry_max_attempts must be > 0".to_string(),
            ));
        }

        // Validate debug flush interval
        if self.debug_flush_interval_secs == 0 {
            return Err(ZerobusError::ConfigurationError(
                "debug_flush_interval_secs must be > 0".to_string(),
            ));
        }

        // Validate retry delay configuration
        if self.retry_max_delay_ms < self.retry_base_delay_ms {
            return Err(ZerobusError::ConfigurationError(format!(
                "retry_max_delay_ms ({}) must be >= retry_base_delay_ms ({})",
                self.retry_max_delay_ms, self.retry_base_delay_ms
            )));
        }

        Ok(())
    }
}

impl OtlpSdkConfig {
    /// Validate the SDK configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if configuration is valid, or `Err(ZerobusError)` if invalid.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if:
    /// - `endpoint` is provided but not a valid URL
    /// - `output_dir` is provided but not a valid path
    /// - `write_interval_secs` is 0
    /// - `log_level` is not a valid log level
    pub fn validate(&self) -> Result<(), ZerobusError> {
        // Validate endpoint URL if provided
        if let Some(endpoint) = &self.endpoint {
            if !endpoint.starts_with("https://") && !endpoint.starts_with("http://") {
                return Err(ZerobusError::ConfigurationError(format!(
                    "endpoint must start with 'https://' or 'http://', got: '{}'",
                    endpoint
                )));
            }
        }

        // Validate output_dir path if provided
        if let Some(output_dir) = &self.output_dir {
            if !output_dir.is_absolute() && !output_dir.is_relative() {
                return Err(ZerobusError::ConfigurationError(format!(
                    "output_dir must be a valid path, got: '{}'",
                    output_dir.display()
                )));
            }
        }

        // Validate write_interval_secs
        if self.write_interval_secs == 0 {
            return Err(ZerobusError::ConfigurationError(
                "write_interval_secs must be > 0".to_string(),
            ));
        }

        // Validate log_level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.to_lowercase().as_str()) {
            return Err(ZerobusError::ConfigurationError(format!(
                "log_level must be one of {:?}, got: '{}'",
                valid_levels, self.log_level
            )));
        }

        Ok(())
    }
}

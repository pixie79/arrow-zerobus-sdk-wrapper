//! Configuration types for Zerobus SDK Wrapper
//!
//! This module defines the configuration structures and validation logic.

use crate::error::ZerobusError;
use secrecy::SecretString;
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

impl Default for OtlpSdkConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            output_dir: None,
            write_interval_secs: 5,
            log_level: "info".to_string(),
        }
    }
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
    /// Stored securely to prevent exposure in memory dumps
    pub client_id: Option<SecretString>,
    /// OAuth2 client secret (required for SDK)
    /// Stored securely to prevent exposure in memory dumps
    pub client_secret: Option<SecretString>,
    /// Target table name in Zerobus (required)
    pub table_name: String,
    /// Enable/disable OpenTelemetry observability (default: false)
    pub observability_enabled: bool,
    /// OpenTelemetry configuration (optional)
    pub observability_config: Option<OtlpSdkConfig>,
    /// Enable/disable debug file output (default: false)
    /// @deprecated Use debug_arrow_enabled and debug_protobuf_enabled instead
    pub debug_enabled: bool,
    /// Enable/disable Arrow debug file output (default: false)
    /// When true, Arrow debug files (.arrows) are written to debug_output_dir
    pub debug_arrow_enabled: bool,
    /// Enable/disable Protobuf debug file output (default: false)
    /// When true, Protobuf debug files (.proto) are written to debug_output_dir
    pub debug_protobuf_enabled: bool,
    /// Output directory for debug files (required if debug_enabled)
    pub debug_output_dir: Option<PathBuf>,
    /// Debug file flush interval in seconds (default: 5)
    pub debug_flush_interval_secs: u64,
    /// Maximum debug file size in bytes before rotation (optional)
    pub debug_max_file_size: Option<u64>,
    /// Maximum number of rotated debug files to retain per type (default: Some(10))
    /// When Some(n), keeps last n rotated files, automatically deleting oldest when limit exceeded
    /// When None, unlimited retention (no automatic cleanup)
    pub debug_max_files_retained: Option<usize>,
    /// Maximum retry attempts for transient failures (default: 5)
    pub retry_max_attempts: u32,
    /// Base delay in milliseconds for exponential backoff (default: 100)
    pub retry_base_delay_ms: u64,
    /// Maximum delay in milliseconds for exponential backoff (default: 30000)
    pub retry_max_delay_ms: u64,
    /// Disable Zerobus SDK transmission while maintaining debug file output (default: false)
    ///
    /// When `true`, the wrapper will skip all Zerobus SDK calls (initialization,
    /// stream creation, data transmission) while still writing debug files (Arrow
    /// and Protobuf) if debug output is enabled.
    ///
    /// # Requirements
    /// - When `true`, `debug_enabled` must also be `true`
    /// - Credentials (`client_id`, `client_secret`) are optional when `true`
    ///
    /// # Use Cases
    /// - Local development without network access
    /// - CI/CD testing without credentials
    /// - Performance testing of conversion logic
    pub zerobus_writer_disabled: bool,
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
            debug_arrow_enabled: false,
            debug_protobuf_enabled: false,
            debug_output_dir: None,
            debug_flush_interval_secs: 5,
            debug_max_file_size: None,
            debug_max_files_retained: Some(10),
            retry_max_attempts: 5,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 30000,
            zerobus_writer_disabled: false,
        }
    }

    /// Set OAuth2 credentials
    ///
    /// # Arguments
    ///
    /// * `client_id` - OAuth2 client ID
    /// * `client_secret` - OAuth2 client secret
    ///
    /// Credentials are stored securely using `SecretString` to prevent exposure in memory dumps.
    pub fn with_credentials(mut self, client_id: String, client_secret: String) -> Self {
        self.client_id = Some(SecretString::new(client_id));
        self.client_secret = Some(SecretString::new(client_secret));
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

    /// Set Arrow debug output enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - If `true`, Arrow debug files (.arrows) will be written to `debug_output_dir`
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arrow_zerobus_sdk_wrapper::WrapperConfiguration;
    /// use std::path::PathBuf;
    ///
    /// let config = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// )
    /// .with_debug_arrow_enabled(true)
    /// .with_debug_output(PathBuf::from("./debug_output"));
    /// ```
    pub fn with_debug_arrow_enabled(mut self, enabled: bool) -> Self {
        self.debug_arrow_enabled = enabled;
        self
    }

    /// Set Protobuf debug output enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - If `true`, Protobuf debug files (.proto) will be written to `debug_output_dir`
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arrow_zerobus_sdk_wrapper::WrapperConfiguration;
    /// use std::path::PathBuf;
    ///
    /// let config = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// )
    /// .with_debug_protobuf_enabled(true)
    /// .with_debug_output(PathBuf::from("./debug_output"));
    /// ```
    pub fn with_debug_protobuf_enabled(mut self, enabled: bool) -> Self {
        self.debug_protobuf_enabled = enabled;
        self
    }

    /// Set debug file retention limit
    ///
    /// # Arguments
    ///
    /// * `max_files` - Maximum number of rotated files to retain per type (default: Some(10), None = unlimited)
    ///   When Some(n), keeps last n rotated files, automatically deleting oldest when limit exceeded.
    ///   When None, unlimited retention (no automatic cleanup).
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arrow_zerobus_sdk_wrapper::WrapperConfiguration;
    /// use std::path::PathBuf;
    ///
    /// // Keep last 20 rotated files per type
    /// let config = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// )
    /// .with_debug_arrow_enabled(true)
    /// .with_debug_output(PathBuf::from("./debug_output"))
    /// .with_debug_max_files_retained(Some(20));
    ///
    /// // Unlimited retention (no automatic cleanup)
    /// let config_unlimited = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// )
    /// .with_debug_arrow_enabled(true)
    /// .with_debug_output(PathBuf::from("./debug_output"))
    /// .with_debug_max_files_retained(None);
    /// ```
    pub fn with_debug_max_files_retained(mut self, max_files: Option<usize>) -> Self {
        self.debug_max_files_retained = max_files;
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

    /// Set writer disabled mode
    ///
    /// # Arguments
    ///
    /// * `disabled` - If `true`, disables Zerobus SDK transmission while maintaining debug output
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arrow_zerobus_sdk_wrapper::WrapperConfiguration;
    /// use std::path::PathBuf;
    ///
    /// let config = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// )
    /// .with_debug_output(PathBuf::from("./debug_output"))
    /// .with_zerobus_writer_disabled(true);
    /// ```
    pub fn with_zerobus_writer_disabled(mut self, disabled: bool) -> Self {
        self.zerobus_writer_disabled = disabled;
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
    /// - `zerobus_writer_disabled` is true but `debug_enabled` is false
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

        // Validate table name: Unity Catalog format (catalog.schema.table, schema.table, or table)
        // Each part must contain only ASCII letters, digits, and underscores (Zerobus requirement)
        // Dots are allowed as separators between catalog, schema, and table name parts
        let parts: Vec<&str> = self.table_name.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            return Err(ZerobusError::ConfigurationError(format!(
                "table_name must be in format 'table', 'schema.table', or 'catalog.schema.table'. Got: '{}'",
                self.table_name
            )));
        }

        for (idx, part) in parts.iter().enumerate() {
            if part.is_empty() {
                let part_name = match idx {
                    0 => "table",
                    1 => "schema",
                    2 => "catalog",
                    _ => "part",
                };
                return Err(ZerobusError::ConfigurationError(format!(
                    "table_name {} part cannot be empty. Got: '{}'",
                    part_name, self.table_name
                )));
            }

            if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                let part_name = match idx {
                    0 => "table",
                    1 => "schema",
                    2 => "catalog",
                    _ => "part",
                };
                return Err(ZerobusError::ConfigurationError(format!(
                    "table_name {} part '{}' must contain only ASCII letters, digits, and underscores (Zerobus requirement). Got: '{}'",
                    part_name, part, self.table_name
                )));
            }
        }

        // Validate debug configuration
        // Check if any debug format is enabled (new flags or legacy flag)
        let any_debug_enabled =
            self.debug_arrow_enabled || self.debug_protobuf_enabled || self.debug_enabled;

        if any_debug_enabled && self.debug_output_dir.is_none() {
            return Err(ZerobusError::ConfigurationError(
                "debug_output_dir is required when any debug format is enabled".to_string(),
            ));
        }

        // Validate writer disabled mode requires at least one debug format enabled
        if self.zerobus_writer_disabled && !any_debug_enabled {
            return Err(ZerobusError::ConfigurationError(
                "At least one debug format must be enabled when zerobus_writer_disabled is true. Use with_debug_arrow_enabled() or with_debug_protobuf_enabled() to enable debug output.".to_string(),
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
        // Note: PathBuf is always either absolute or relative, so we just check if it's empty
        if let Some(output_dir) = &self.output_dir {
            if output_dir.as_os_str().is_empty() {
                return Err(ZerobusError::ConfigurationError(
                    "output_dir must not be empty".to_string(),
                ));
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

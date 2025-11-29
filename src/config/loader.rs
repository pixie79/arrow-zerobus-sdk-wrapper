//! Configuration loader for Zerobus SDK Wrapper
//!
//! This module handles loading configuration from YAML files and environment variables.

use crate::config::WrapperConfiguration;
use crate::error::ZerobusError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// YAML configuration structure (for deserialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigYaml {
    pub zerobus_endpoint: Option<String>,
    pub unity_catalog_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub table_name: Option<String>,
    pub observability: Option<ObservabilityYaml>,
    pub debug: Option<DebugYaml>,
    pub retry: Option<RetryYaml>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityYaml {
    pub enabled: Option<bool>,
    pub endpoint: Option<String>,
    pub output_dir: Option<String>,
    pub write_interval_secs: Option<u64>,
    pub log_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugYaml {
    pub enabled: Option<bool>,
    pub output_dir: Option<String>,
    pub flush_interval_secs: Option<u64>,
    pub max_file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryYaml {
    pub max_attempts: Option<u32>,
    pub base_delay_ms: Option<u64>,
    pub max_delay_ms: Option<u64>,
}

/// Load configuration from YAML file
///
/// # Arguments
///
/// * `path` - Path to YAML configuration file
///
/// # Returns
///
/// Returns `WrapperConfiguration` if successful, or `ZerobusError` if loading fails.
pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<WrapperConfiguration, ZerobusError> {
    let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
        ZerobusError::ConfigurationError(format!(
            "Failed to read config file {}: {}",
            path.as_ref().display(),
            e
        ))
    })?;

    let yaml: ConfigYaml = serde_yaml::from_str(&content)
        .map_err(|e| ZerobusError::ConfigurationError(format!("Failed to parse YAML: {}", e)))?;

    let mut config = WrapperConfiguration::new(
        yaml.zerobus_endpoint
            .ok_or_else(|| {
                ZerobusError::ConfigurationError("zerobus_endpoint is required".to_string())
            })?
            .clone(),
        yaml.table_name
            .ok_or_else(|| ZerobusError::ConfigurationError("table_name is required".to_string()))?
            .clone(),
    );

    if let Some(url) = yaml.unity_catalog_url {
        config = config.with_unity_catalog(url);
    }

    if let Some(client_id) = yaml.client_id {
        if let Some(client_secret) = yaml.client_secret {
            config = config.with_credentials(client_id, client_secret);
        }
    }

    if let Some(obs) = yaml.observability {
        if obs.enabled.unwrap_or(false) {
            use crate::config::OtlpSdkConfig;
            let otlp_config = OtlpSdkConfig {
                endpoint: obs.endpoint,
                output_dir: obs.output_dir.map(std::path::PathBuf::from),
                write_interval_secs: obs.write_interval_secs.unwrap_or(5),
                log_level: obs.log_level.unwrap_or_else(|| "info".to_string()),
            };
            config = config.with_observability(otlp_config);
        }
    }

    if let Some(debug) = yaml.debug {
        if debug.enabled.unwrap_or(false) {
            if let Some(output_dir) = debug.output_dir {
                config = config.with_debug_output(std::path::PathBuf::from(output_dir));
                if let Some(interval) = debug.flush_interval_secs {
                    config.debug_flush_interval_secs = interval;
                }
                config.debug_max_file_size = debug.max_file_size;
            }
        }
    }

    if let Some(retry) = yaml.retry {
        if let (Some(max), Some(base), Some(max_delay)) =
            (retry.max_attempts, retry.base_delay_ms, retry.max_delay_ms)
        {
            config = config.with_retry_config(max, base, max_delay);
        }
    }

    config.validate()?;
    Ok(config)
}

/// Load configuration from environment variables
///
/// Reads configuration from environment variables with the following prefixes:
/// - `ZEROBUS_` for Zerobus-specific settings
/// - `OTLP_` for OpenTelemetry settings
/// - `DEBUG_` for debug file settings
/// - `RETRY_` for retry settings
///
/// # Returns
///
/// Returns `WrapperConfiguration` if successful, or `ZerobusError` if loading fails.
pub fn load_from_env() -> Result<WrapperConfiguration, ZerobusError> {
    let endpoint = std::env::var("ZEROBUS_ENDPOINT").map_err(|_| {
        ZerobusError::ConfigurationError(
            "ZEROBUS_ENDPOINT environment variable is required".to_string(),
        )
    })?;

    let table_name = std::env::var("ZEROBUS_TABLE_NAME").map_err(|_| {
        ZerobusError::ConfigurationError(
            "ZEROBUS_TABLE_NAME environment variable is required".to_string(),
        )
    })?;

    let mut config = WrapperConfiguration::new(endpoint, table_name);

    if let Ok(url) = std::env::var("UNITY_CATALOG_URL") {
        config = config.with_unity_catalog(url);
    }

    if let (Ok(client_id), Ok(client_secret)) = (
        std::env::var("ZEROBUS_CLIENT_ID"),
        std::env::var("ZEROBUS_CLIENT_SECRET"),
    ) {
        config = config.with_credentials(client_id, client_secret);
    }

    if std::env::var("OTLP_ENABLED").unwrap_or_default() == "true" {
        use crate::config::OtlpSdkConfig;
        let otlp_config = OtlpSdkConfig {
            endpoint: std::env::var("OTLP_ENDPOINT").ok(),
            output_dir: std::env::var("OTLP_OUTPUT_DIR")
                .ok()
                .map(std::path::PathBuf::from),
            write_interval_secs: std::env::var("OTLP_WRITE_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            log_level: std::env::var("OTLP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        };
        config = config.with_observability(otlp_config);
    }

    if std::env::var("DEBUG_ENABLED").unwrap_or_default() == "true" {
        if let Ok(output_dir) = std::env::var("DEBUG_OUTPUT_DIR") {
            config = config.with_debug_output(std::path::PathBuf::from(output_dir));
            if let Ok(interval) = std::env::var("DEBUG_FLUSH_INTERVAL_SECS") {
                config.debug_flush_interval_secs = interval.parse().unwrap_or(5);
            }
            if let Ok(max_size) = std::env::var("DEBUG_MAX_FILE_SIZE") {
                config.debug_max_file_size = max_size.parse().ok();
            }
        }
    }

    if let (Ok(max), Ok(base), Ok(max_delay)) = (
        std::env::var("RETRY_MAX_ATTEMPTS"),
        std::env::var("RETRY_BASE_DELAY_MS"),
        std::env::var("RETRY_MAX_DELAY_MS"),
    ) {
        if let (Ok(max_u32), Ok(base_u64), Ok(max_delay_u64)) = (
            max.parse::<u32>(),
            base.parse::<u64>(),
            max_delay.parse::<u64>(),
        ) {
            config = config.with_retry_config(max_u32, base_u64, max_delay_u64);
        }
    }

    config.validate()?;
    Ok(config)
}

//! Unit tests for configuration types

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, OtlpConfig, OtlpSdkConfig};
use std::path::PathBuf;

#[test]
fn test_config_new() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    assert_eq!(config.zerobus_endpoint, "https://test.cloud.databricks.com");
    assert_eq!(config.table_name, "test_table");
    assert!(!config.observability_enabled);
    assert!(!config.debug_enabled);
    assert_eq!(config.retry_max_attempts, 5);
    assert_eq!(config.debug_flush_interval_secs, 5);
}

#[test]
fn test_config_with_credentials() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string());

    assert_eq!(config.client_id, Some("client_id".to_string()));
    assert_eq!(config.client_secret, Some("client_secret".to_string()));
}

#[test]
fn test_config_with_unity_catalog() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_unity_catalog("https://unity-catalog-url".to_string());

    assert_eq!(
        config.unity_catalog_url,
        Some("https://unity-catalog-url".to_string())
    );
}

#[test]
fn test_config_with_observability() {
    let otlp_config = OtlpSdkConfig {
        endpoint: Some("http://localhost:4317".to_string()),
        output_dir: Some(PathBuf::from("/tmp/otlp")),
        write_interval_secs: 5,
        log_level: "info".to_string(),
    };

    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_observability(otlp_config);

    assert!(config.observability_enabled);
    assert!(config.observability_config.is_some());
}

#[test]
fn test_otlp_sdk_config_default() {
    let config = OtlpSdkConfig {
        endpoint: None,
        output_dir: None,
        write_interval_secs: 5,
        log_level: "info".to_string(),
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_otlp_sdk_config_validate_valid_endpoint() {
    let config = OtlpSdkConfig {
        endpoint: Some("https://otlp.example.com".to_string()),
        output_dir: None,
        write_interval_secs: 5,
        log_level: "info".to_string(),
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_otlp_sdk_config_validate_invalid_endpoint() {
    let config = OtlpSdkConfig {
        endpoint: Some("invalid-url".to_string()),
        output_dir: None,
        write_interval_secs: 5,
        log_level: "info".to_string(),
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_otlp_sdk_config_validate_valid_output_dir() {
    let config = OtlpSdkConfig {
        endpoint: None,
        output_dir: Some(PathBuf::from("/tmp/otlp")),
        write_interval_secs: 5,
        log_level: "info".to_string(),
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_otlp_sdk_config_validate_zero_write_interval() {
    let config = OtlpSdkConfig {
        endpoint: None,
        output_dir: None,
        write_interval_secs: 0,
        log_level: "info".to_string(),
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_otlp_sdk_config_validate_invalid_log_level() {
    let config = OtlpSdkConfig {
        endpoint: None,
        output_dir: None,
        write_interval_secs: 5,
        log_level: "invalid".to_string(),
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_otlp_sdk_config_validate_valid_log_levels() {
    let valid_levels = ["trace", "debug", "info", "warn", "error"];

    for level in valid_levels {
        let config = OtlpSdkConfig {
            endpoint: None,
            output_dir: None,
            write_interval_secs: 5,
            log_level: level.to_string(),
        };

        assert!(config.validate().is_ok(), "Log level '{}' should be valid", level);
    }
}

#[test]
fn test_config_with_debug_output() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(PathBuf::from("/tmp/debug"));

    assert!(config.debug_enabled);
    assert_eq!(
        config.debug_output_dir,
        Some(PathBuf::from("/tmp/debug"))
    );
}

#[test]
fn test_config_with_retry_config() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_retry_config(10, 200, 60000);

    assert_eq!(config.retry_max_attempts, 10);
    assert_eq!(config.retry_base_delay_ms, 200);
    assert_eq!(config.retry_max_delay_ms, 60000);
}

#[test]
fn test_config_validate_success() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_invalid_endpoint() {
    let config = WrapperConfiguration::new(
        "invalid-endpoint".to_string(),
        "test_table".to_string(),
    );

    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_debug_enabled_no_dir() {
    let mut config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );
    config.debug_enabled = true;
    config.debug_output_dir = None;

    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_retry_attempts() {
    let mut config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );
    config.retry_max_attempts = 0;

    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_flush_interval() {
    let mut config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );
    config.debug_flush_interval_secs = 0;

    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_max_delay_less_than_base() {
    let mut config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );
    config.retry_base_delay_ms = 1000;
    config.retry_max_delay_ms = 500;

    assert!(config.validate().is_err());
}


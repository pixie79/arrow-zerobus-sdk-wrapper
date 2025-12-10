//! Integration tests for configuration

use arrow_zerobus_sdk_wrapper::config::loader;
use arrow_zerobus_sdk_wrapper::WrapperConfiguration;
use std::fs;
use tempfile::TempDir;

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

    use secrecy::ExposeSecret;
    assert_eq!(
        config
            .client_id
            .as_ref()
            .map(|s| s.expose_secret().as_str()),
        Some("client_id")
    );
    assert_eq!(
        config
            .client_secret
            .as_ref()
            .map(|s| s.expose_secret().as_str()),
        Some("client_secret")
    );
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
    let config =
        WrapperConfiguration::new("invalid-endpoint".to_string(), "test_table".to_string());

    assert!(config.validate().is_err());
}

#[test]
fn test_load_from_yaml_success() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
zerobus_endpoint: https://test.cloud.databricks.com
table_name: test_table
unity_catalog_url: https://unity-catalog-url
client_id: test_client_id
client_secret: test_client_secret
"#;

    fs::write(&yaml_path, yaml_content).unwrap();

    let config = loader::load_from_yaml(&yaml_path).unwrap();

    assert_eq!(config.zerobus_endpoint, "https://test.cloud.databricks.com");
    assert_eq!(config.table_name, "test_table");
}

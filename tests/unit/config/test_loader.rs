//! Unit tests for configuration loader

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, config::loader};
use std::fs;
use tempfile::TempDir;

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
    assert_eq!(
        config.unity_catalog_url,
        Some("https://unity-catalog-url".to_string())
    );
    use secrecy::ExposeSecret;
    assert_eq!(
        config.client_id.as_ref().map(|s| s.expose_secret().as_str()),
        Some("test_client_id")
    );
    assert_eq!(
        config.client_secret.as_ref().map(|s| s.expose_secret().as_str()),
        Some("test_client_secret")
    );
}

#[test]
fn test_load_from_yaml_missing_endpoint() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
table_name: test_table
"#;

    fs::write(&yaml_path, yaml_content).unwrap();

    let result = loader::load_from_yaml(&yaml_path);
    assert!(result.is_err());
}

#[test]
fn test_load_from_yaml_with_observability() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
zerobus_endpoint: https://test.cloud.databricks.com
table_name: test_table
observability:
  enabled: true
  endpoint: http://localhost:4317
"#;

    fs::write(&yaml_path, yaml_content).unwrap();

    let config = loader::load_from_yaml(&yaml_path).unwrap();

    assert!(config.observability_enabled);
    assert!(config.observability_config.is_some());
}

#[test]
fn test_load_from_yaml_with_debug() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
zerobus_endpoint: https://test.cloud.databricks.com
table_name: test_table
debug:
  enabled: true
  output_dir: /tmp/debug
  flush_interval_secs: 10
  max_file_size: 10485760
"#;

    fs::write(&yaml_path, yaml_content).unwrap();

    let config = loader::load_from_yaml(&yaml_path).unwrap();

    assert!(config.debug_enabled);
    assert_eq!(
        config.debug_output_dir,
        Some(std::path::PathBuf::from("/tmp/debug"))
    );
    assert_eq!(config.debug_flush_interval_secs, 10);
    assert_eq!(config.debug_max_file_size, Some(10485760));
}

#[test]
fn test_load_from_yaml_with_retry() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
zerobus_endpoint: https://test.cloud.databricks.com
table_name: test_table
retry:
  max_attempts: 10
  base_delay_ms: 200
  max_delay_ms: 60000
"#;

    fs::write(&yaml_path, yaml_content).unwrap();

    let config = loader::load_from_yaml(&yaml_path).unwrap();

    assert_eq!(config.retry_max_attempts, 10);
    assert_eq!(config.retry_base_delay_ms, 200);
    assert_eq!(config.retry_max_delay_ms, 60000);
}

#[test]
fn test_load_from_env() {
    std::env::set_var("ZEROBUS_ENDPOINT", "https://test.cloud.databricks.com");
    std::env::set_var("ZEROBUS_TABLE_NAME", "test_table");
    std::env::set_var("UNITY_CATALOG_URL", "https://unity-catalog-url");
    std::env::set_var("ZEROBUS_CLIENT_ID", "test_client_id");
    std::env::set_var("ZEROBUS_CLIENT_SECRET", "test_client_secret");

    let config = loader::load_from_env().unwrap();

    assert_eq!(config.zerobus_endpoint, "https://test.cloud.databricks.com");
    assert_eq!(config.table_name, "test_table");
    assert_eq!(
        config.unity_catalog_url,
        Some("https://unity-catalog-url".to_string())
    );
    use secrecy::ExposeSecret;
    assert_eq!(
        config.client_id.as_ref().map(|s| s.expose_secret().as_str()),
        Some("test_client_id")
    );
    assert_eq!(
        config.client_secret.as_ref().map(|s| s.expose_secret().as_str()),
        Some("test_client_secret")
    );

    // Cleanup
    std::env::remove_var("ZEROBUS_ENDPOINT");
    std::env::remove_var("ZEROBUS_TABLE_NAME");
    std::env::remove_var("UNITY_CATALOG_URL");
    std::env::remove_var("ZEROBUS_CLIENT_ID");
    std::env::remove_var("ZEROBUS_CLIENT_SECRET");
}

#[test]
fn test_load_from_env_missing_required() {
    std::env::remove_var("ZEROBUS_ENDPOINT");
    std::env::remove_var("ZEROBUS_TABLE_NAME");

    let result = loader::load_from_env();
    assert!(result.is_err());
}


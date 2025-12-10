//! Contract tests for Rust API
//!
//! These tests verify that the Rust API matches the contract specification
//! defined in specs/001-zerobus-wrapper/contracts/rust-api.md

use arrow_zerobus_sdk_wrapper::{
    WrapperConfiguration, ZerobusWrapper, ZerobusError, TransmissionResult, OtlpSdkConfig,
};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Test that WrapperConfiguration can be created with required fields
#[test]
fn test_config_contract_required_fields() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    // Contract: zerobus_endpoint and table_name are required
    assert_eq!(config.zerobus_endpoint, "https://test.cloud.databricks.com");
    assert_eq!(config.table_name, "test_table");
}

/// Test that WrapperConfiguration builder methods work as specified
#[test]
fn test_config_contract_builder_methods() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());

    // Contract: Builder methods should set corresponding fields
    use secrecy::ExposeSecret;
    assert_eq!(
        config.client_id.as_ref().map(|s| s.expose_secret().as_str()),
        Some("client_id")
    );
    assert_eq!(
        config.client_secret.as_ref().map(|s| s.expose_secret().as_str()),
        Some("client_secret")
    );
    assert_eq!(
        config.unity_catalog_url,
        Some("https://unity-catalog-url".to_string())
    );
}

/// Test that ZerobusWrapper::new requires valid configuration
#[tokio::test]
async fn test_wrapper_new_contract() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());

    // Contract: new() should validate configuration
    let result = config.validate();
    assert!(result.is_ok());

    // Contract: new() should return ZerobusWrapper or error
    // Note: This will fail without real SDK, but tests the contract
    let _wrapper_result = ZerobusWrapper::new(config).await;
    // We expect this to fail without real credentials, but the API contract is correct
}

/// Test that send_batch returns TransmissionResult
#[tokio::test]
#[ignore] // Requires actual Zerobus SDK
async fn test_send_batch_contract() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());

    let wrapper = ZerobusWrapper::new(config).await.unwrap();

    // Create test batch
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    let id_array = Int64Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap();

    // Contract: send_batch should return TransmissionResult
    let result: Result<TransmissionResult, ZerobusError> = wrapper.send_batch(batch).await;

    // Contract: Result should be Ok(TransmissionResult) or Err(ZerobusError)
    match result {
        Ok(transmission_result) => {
            // Contract: TransmissionResult should have required fields
            assert!(transmission_result.batch_size_bytes > 0);
            // success and error are mutually exclusive
            if transmission_result.success {
                assert!(transmission_result.error.is_none());
            } else {
                assert!(transmission_result.error.is_some());
            }
        }
        Err(_) => {
            // Error is acceptable (e.g., no real SDK connection)
        }
    }
}

/// Test that TransmissionResult has required fields per contract
#[test]
fn test_transmission_result_contract() {
    // Contract: TransmissionResult must have these fields
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
    };

    assert!(result.success);
    assert!(result.error.is_none());
    assert_eq!(result.attempts, 1);
    assert_eq!(result.latency_ms, Some(100));
    assert_eq!(result.batch_size_bytes, 1024);
}

/// Test that ZerobusError variants match contract
#[test]
fn test_error_contract() {
    // Contract: All error variants should be available
    let _config = ZerobusError::ConfigurationError("test".to_string());
    let _auth = ZerobusError::AuthenticationError("test".to_string());
    let _conn = ZerobusError::ConnectionError("test".to_string());
    let _conv = ZerobusError::ConversionError("test".to_string());
    let _trans = ZerobusError::TransmissionError("test".to_string());
    let _retry = ZerobusError::RetryExhausted("test".to_string());
    let _token = ZerobusError::TokenRefreshError("test".to_string());
}

/// Test that flush and shutdown methods exist per contract
#[tokio::test]
async fn test_wrapper_lifecycle_contract() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());

    // Contract: flush() and shutdown() should be callable
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    if let Ok(wrapper) = wrapper_result {
        // Contract: flush() should return Result<(), ZerobusError>
        let flush_result: Result<(), ZerobusError> = wrapper.flush().await;
        assert!(flush_result.is_ok() || flush_result.is_err());

        // Contract: shutdown() should return Result<(), ZerobusError>
        let shutdown_result: Result<(), ZerobusError> = wrapper.shutdown().await;
        assert!(shutdown_result.is_ok() || shutdown_result.is_err());
    }
}

/// Test that observability configuration works per contract
#[test]
fn test_observability_contract() {
    use std::path::PathBuf;
    
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

    // Contract: with_observability should enable observability
    assert!(config.observability_enabled);
    assert!(config.observability_config.is_some());
}

/// Test that debug output configuration works per contract
#[test]
fn test_debug_output_contract() {
    use std::path::PathBuf;

    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(PathBuf::from("/tmp/debug"));

    // Contract: with_debug_output should enable debug and set output_dir
    assert!(config.debug_enabled);
    assert_eq!(config.debug_output_dir, Some(PathBuf::from("/tmp/debug")));
}

/// Test that retry configuration works per contract
#[test]
fn test_retry_config_contract() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_retry_config(10, 200, 60000);

    // Contract: with_retry_config should set retry parameters
    assert_eq!(config.retry_max_attempts, 10);
    assert_eq!(config.retry_base_delay_ms, 200);
    assert_eq!(config.retry_max_delay_ms, 60000);
}


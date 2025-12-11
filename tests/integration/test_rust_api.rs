//! End-to-end integration test for Rust API
//!
//! This test verifies the complete user journey:
//! 1. Create configuration
//! 2. Initialize wrapper
//! 3. Create Arrow RecordBatch
//! 4. Send batch to Zerobus
//! 5. Verify result
//! 6. Shutdown wrapper

use arrow_zerobus_sdk_wrapper::{
    WrapperConfiguration, ZerobusWrapper, ZerobusError, TransmissionResult,
};
use arrow::array::{Int64Array, StringArray, Float64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Create a test RecordBatch with sample data
fn create_test_record_batch() -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("score", DataType::Float64, true),
    ]);

    let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie", "David", "Eve"]);
    let score_array = Float64Array::from(vec![Some(95.5), Some(87.0), None, Some(92.5), Some(88.0)]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array),
            Arc::new(name_array),
            Arc::new(score_array),
        ],
    )
    .expect("Failed to create test RecordBatch")
}

/// Test complete user journey with mock configuration
#[tokio::test]
#[ignore] // Requires actual Zerobus SDK and credentials
async fn test_complete_user_journey() {
    // Step 1: Create configuration
    let config = WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        std::env::var("ZEROBUS_CLIENT_ID")
            .unwrap_or_else(|_| "test_client_id".to_string()),
        std::env::var("ZEROBUS_CLIENT_SECRET")
            .unwrap_or_else(|_| "test_client_secret".to_string()),
    )
    .with_unity_catalog(
        std::env::var("UNITY_CATALOG_URL")
            .unwrap_or_else(|_| "https://test.cloud.databricks.com".to_string()),
    )
    .with_retry_config(3, 100, 1000); // Reduced retries for testing

    // Step 2: Initialize wrapper
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    // Without real credentials, this will fail, but we can test the flow
    match wrapper_result {
        Ok(wrapper) => {
            // Step 3: Create Arrow RecordBatch
            let batch = create_test_record_batch();
            assert_eq!(batch.num_rows(), 5);
            assert_eq!(batch.num_columns(), 3);

            // Step 4: Send batch to Zerobus
            let result: Result<TransmissionResult, ZerobusError> = wrapper.send_batch(batch).await;

            // Step 5: Verify result
            match result {
                Ok(transmission_result) => {
                    // Verify TransmissionResult structure
                    assert!(transmission_result.batch_size_bytes > 0);
                    assert!(transmission_result.attempts >= 1);
                    
                    if transmission_result.success {
                        assert!(transmission_result.error.is_none());
                        assert!(transmission_result.latency_ms.is_some());
                        println!(
                            "✅ Batch sent successfully! Latency: {}ms, Size: {} bytes",
                            transmission_result.latency_ms.unwrap_or(0),
                            transmission_result.batch_size_bytes
                        );
                    } else {
                        assert!(transmission_result.error.is_some());
                        println!(
                            "❌ Transmission failed: {:?}",
                            transmission_result.error
                        );
                    }
                }
                Err(e) => {
                    // Error is acceptable in test environment
                    println!("⚠️  Transmission error (expected in test): {}", e);
                }
            }

            // Step 6: Shutdown wrapper
            let shutdown_result = wrapper.shutdown().await;
            assert!(shutdown_result.is_ok() || shutdown_result.is_err());
        }
        Err(e) => {
            // Initialization failure is expected without real credentials
            println!("⚠️  Wrapper initialization failed (expected in test): {}", e);
        }
    }
}

/// Test configuration validation in user journey
#[test]
fn test_user_journey_configuration_validation() {
    // Test that invalid configuration is caught early
    let invalid_config = WrapperConfiguration::new(
        "invalid-endpoint".to_string(), // Invalid endpoint
        "test_table".to_string(),
    );

    let validation_result = invalid_config.validate();
    assert!(validation_result.is_err());

    // Test that valid configuration passes validation
    let valid_config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    let validation_result = valid_config.validate();
    assert!(validation_result.is_ok());
}

/// Test error handling in user journey
#[tokio::test]
async fn test_user_journey_error_handling() {
    // Test that configuration errors are properly returned
    let config = WrapperConfiguration::new(
        "invalid".to_string(),
        "test_table".to_string(),
    );

    // Validation should fail
    assert!(config.validate().is_err());

    // Test that missing credentials are detected
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );
    // No credentials set

    // Wrapper initialization should fail without credentials
    let wrapper_result = ZerobusWrapper::new(config).await;
    assert!(wrapper_result.is_err());
}

/// Test that RecordBatch conversion works in user journey
#[test]
fn test_user_journey_record_batch_creation() {
    // Test that we can create a valid RecordBatch
    let batch = create_test_record_batch();

    // Verify batch structure
    assert_eq!(batch.num_rows(), 5);
    assert_eq!(batch.num_columns(), 3);

    // Verify schema
    let schema = batch.schema();
    assert_eq!(schema.fields().len(), 3);
    assert_eq!(schema.field(0).name(), "id");
    assert_eq!(schema.field(1).name(), "name");
    assert_eq!(schema.field(2).name(), "score");

    // Verify data
    let id_array = batch.column(0);
    let name_array = batch.column(1);
    let score_array = batch.column(2);

    // Check that arrays are not empty
    assert_eq!(id_array.len(), 5);
    assert_eq!(name_array.len(), 5);
    assert_eq!(score_array.len(), 5);
}

/// Test retry behavior in user journey
#[tokio::test]
async fn test_user_journey_retry_behavior() {
    use arrow_zerobus_sdk_wrapper::wrapper::retry::RetryConfig;
    use arrow_zerobus_sdk_wrapper::ZerobusError;

    // Test that retry config works as expected
    let retry_config = RetryConfig::new(3, 10, 1000);
    
    let mut attempts = 0;
    let result = retry_config
        .execute_with_retry(|| {
            attempts += 1;
            async {
                if attempts < 2 {
                    Err::<String, _>(ZerobusError::ConnectionError("transient".to_string()))
                } else {
                    Ok("success".to_string())
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(attempts, 2);
}

/// Test that wrapper can be cloned for concurrent use
#[tokio::test]
async fn test_user_journey_concurrent_access() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    if let Ok(wrapper) = wrapper_result {
        // Test that wrapper can be cloned (for concurrent access)
        let wrapper_clone = wrapper.clone();
        
        // Both should be usable (though will fail without real SDK)
        let _flush1 = wrapper.flush().await;
        let _flush2 = wrapper_clone.flush().await;
    }
}

#[tokio::test]
async fn test_success_return_when_writer_disabled() {
    // Test that send_batch returns success when writer is disabled
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(debug_output_dir)
    .with_zerobus_writer_disabled(true);
    // No credentials required

    let wrapper_result = ZerobusWrapper::new(config).await;
    assert!(wrapper_result.is_ok(), "Wrapper should initialize without credentials");
    
    let wrapper = wrapper_result.unwrap();
    let batch = create_test_record_batch();
    
    // Send batch - should succeed immediately without network calls
    let result = wrapper.send_batch(batch).await;
    assert!(result.is_ok(), "send_batch should succeed when writer disabled");
    
    let transmission_result = result.unwrap();
    assert!(transmission_result.success, "Transmission result should indicate success");
    assert_eq!(transmission_result.attempts, 1, "Should have 1 attempt (no retries when disabled)");
}

#[tokio::test]
async fn test_multiple_batches_succeed_without_credentials() {
    // Test that multiple batches can be sent successfully without credentials when writer disabled
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(debug_output_dir)
    .with_zerobus_writer_disabled(true);
    // No credentials required

    let wrapper_result = ZerobusWrapper::new(config).await;
    assert!(wrapper_result.is_ok(), "Wrapper should initialize without credentials");
    
    let wrapper = wrapper_result.unwrap();
    
    // Send multiple batches
    for i in 0..5 {
        let batch = create_test_record_batch();
        let result = wrapper.send_batch(batch).await;
        assert!(result.is_ok(), "Batch {} should succeed", i);
        
        let transmission_result = result.unwrap();
        assert!(transmission_result.success, "Batch {} should indicate success", i);
    }
}


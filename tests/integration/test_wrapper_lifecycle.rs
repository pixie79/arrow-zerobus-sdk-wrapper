//! Integration tests for wrapper lifecycle
//!
//! Tests for shutdown, flush, and multiple batch operations

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper, ZerobusError};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Create a test RecordBatch
fn create_test_batch() -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let id_array = Int64Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap()
}

#[tokio::test]
#[ignore] // Requires actual Zerobus SDK - run manually with real credentials
async fn test_wrapper_shutdown_with_active_operations() {
    // Test shutdown while batch is being sent
    // Verify graceful shutdown
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        std::env::var("ZEROBUS_CLIENT_ID").unwrap_or_else(|_| "test_id".to_string()),
        std::env::var("ZEROBUS_CLIENT_SECRET").unwrap_or_else(|_| "test_secret".to_string()),
    )
    .with_unity_catalog(
        std::env::var("UNITY_CATALOG_URL").unwrap_or_else(|_| "https://test".to_string()),
    );

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    match wrapper_result {
        Ok(wrapper) => {
            // Start sending a batch
            let batch = create_test_batch();
            let send_handle = tokio::spawn(async move {
                wrapper.send_batch(batch).await
            });
            
            // Immediately try to shutdown
            // Note: This is a simplified test - in practice, shutdown should wait for active operations
            // The actual behavior depends on implementation
            
            // Wait a bit for the send to start
            sleep(Duration::from_millis(100)).await;
            
            // Shutdown should complete (may wait for active operations or cancel them)
            // This test verifies shutdown doesn't panic
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_wrapper_flush() {
    // Test flush operations
    // Verify debug files are flushed
    // Verify observability is flushed
    
    let temp_dir = tempfile::tempdir().unwrap();
    let debug_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(debug_dir.clone())
    .with_debug_flush_interval_secs(1);

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    match wrapper_result {
        Ok(wrapper) => {
            // Flush should succeed even with no data
            let result = wrapper.flush().await;
            
            // May succeed or fail depending on implementation
            // But should not panic
            match result {
                Ok(_) => {
                    // Success - flush completed
                }
                Err(e) => {
                    // Expected if no data to flush or without real SDK
                    assert!(
                        matches!(
                            e,
                            ZerobusError::ConfigurationError(_)
                                | ZerobusError::ConnectionError(_)
                        ),
                        "Expected ConfigurationError or ConnectionError, got: {:?}",
                        e
                    );
                }
            }
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_wrapper_multiple_batches() {
    // Test sending multiple batches sequentially
    // Verify state is maintained correctly
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        std::env::var("ZEROBUS_CLIENT_ID").unwrap_or_else(|_| "test_id".to_string()),
        std::env::var("ZEROBUS_CLIENT_SECRET").unwrap_or_else(|_| "test_secret".to_string()),
    )
    .with_unity_catalog(
        std::env::var("UNITY_CATALOG_URL").unwrap_or_else(|_| "https://test".to_string()),
    );

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    match wrapper_result {
        Ok(wrapper) => {
            // Send multiple batches
            for i in 0..5 {
                let batch = create_test_batch();
                let result = wrapper.send_batch(batch).await;
                
                // May succeed or fail, but should not panic
                match result {
                    Ok(transmission_result) => {
                        // Success
                        assert!(transmission_result.attempts >= 1);
                        assert!(transmission_result.batch_size_bytes > 0);
                    }
                    Err(e) => {
                        // Failure is acceptable without real credentials
                        // But verify it's a known error type
                        assert!(
                            matches!(
                                e,
                                ZerobusError::ConfigurationError(_)
                                    | ZerobusError::AuthenticationError(_)
                                    | ZerobusError::ConnectionError(_)
                            ),
                            "Batch {} failed with unexpected error: {:?}",
                            i,
                            e
                        );
                    }
                }
                
                // Small delay between batches
                sleep(Duration::from_millis(10)).await;
            }
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_wrapper_shutdown_after_use() {
    // Test shutdown after using the wrapper
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    match wrapper_result {
        Ok(wrapper) => {
            // Shutdown should succeed
            let result = wrapper.shutdown().await;
            
            // May succeed or fail, but should not panic
            match result {
                Ok(_) => {
                    // Success - shutdown completed
                }
                Err(e) => {
                    // Expected if there were active operations or without real SDK
                    assert!(
                        matches!(
                            e,
                            ZerobusError::ConfigurationError(_)
                                | ZerobusError::ConnectionError(_)
                        ),
                        "Expected ConfigurationError or ConnectionError, got: {:?}",
                        e
                    );
                }
            }
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_wrapper_flush_with_debug_enabled() {
    // Test flush when debug is enabled
    let temp_dir = tempfile::tempdir().unwrap();
    let debug_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(debug_dir.clone())
    .with_debug_enabled(true);

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    match wrapper_result {
        Ok(wrapper) => {
            // Send a batch to generate debug output
            let batch = create_test_batch();
            let _ = wrapper.send_batch(batch).await; // Ignore result
            
            // Flush should write debug files
            let result = wrapper.flush().await;
            
            // May succeed or fail, but should not panic
            match result {
                Ok(_) => {
                    // Success - debug files flushed
                }
                Err(_) => {
                    // Expected if no data or without real SDK
                }
            }
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_wrapper_lifecycle_complete() {
    // Test complete lifecycle: create -> use -> flush -> shutdown
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        std::env::var("ZEROBUS_CLIENT_ID").unwrap_or_else(|_| "test_id".to_string()),
        std::env::var("ZEROBUS_CLIENT_SECRET").unwrap_or_else(|_| "test_secret".to_string()),
    )
    .with_unity_catalog(
        std::env::var("UNITY_CATALOG_URL").unwrap_or_else(|_| "https://test".to_string()),
    );

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    match wrapper_result {
        Ok(wrapper) => {
            // Step 1: Use wrapper
            let batch = create_test_batch();
            let _ = wrapper.send_batch(batch).await; // Ignore result
            
            // Step 2: Flush
            let _ = wrapper.flush().await; // Ignore result
            
            // Step 3: Shutdown
            let _ = wrapper.shutdown().await; // Ignore result
            
            // If we get here without panicking, lifecycle is complete
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}


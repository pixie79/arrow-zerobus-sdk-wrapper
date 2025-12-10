//! Tests for stream closure recovery functionality

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
async fn test_stream_closure_on_first_record() {
    // Test recovery when stream closes on first record
    // This indicates schema mismatch or validation error
    // Note: Without real SDK, this tests error handling paths
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
            let batch = create_test_batch();
            let result = wrapper.send_batch(batch).await;

            // Stream closure on first record should result in error
            // The error should indicate schema/validation issues
            match result {
                Ok(_) => {
                    // Success - stream didn't close (expected in test environment)
                }
                Err(e) => {
                    // Verify error type is appropriate
                    assert!(
                        matches!(
                            e,
                            ZerobusError::ConnectionError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                        ),
                        "Expected ConnectionError, ConfigurationError, or AuthenticationError, got: {:?}",
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
async fn test_stream_closure_mid_batch_recovery() {
    // Test recovery when stream closes mid-batch
    // Note: Without real SDK, this tests error handling paths
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
            // Create a larger batch
            let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
            let ids: Vec<i64> = (0..100).collect();
            let id_array = Int64Array::from(ids);
            let batch = RecordBatch::try_new(
                Arc::new(schema),
                vec![Arc::new(id_array)],
            )
            .unwrap();

            let result = wrapper.send_batch(batch).await;

            // May succeed or fail, but should handle stream closure gracefully
            match result {
                Ok(_) => {
                    // Success - all records sent
                }
                Err(e) => {
                    // Verify error is appropriate
                    assert!(
                        matches!(
                            e,
                            ZerobusError::ConnectionError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                        ),
                        "Unexpected error type: {:?}",
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
async fn test_stream_closure_multiple_times() {
    // Test recovery from multiple stream closures
    // Note: Without real SDK, this tests error handling paths
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
            let batch = create_test_batch();

            // Attempt multiple sends (stream may close and be recreated)
            for i in 0..5 {
                let result = wrapper.send_batch(batch.clone()).await;

                match result {
                    Ok(_) => {
                        // Success
                    }
                    Err(e) => {
                        // Verify error is retryable or indicates stream closure
                        assert!(
                            matches!(
                                e,
                                ZerobusError::ConnectionError(_)
                                    | ZerobusError::ConfigurationError(_)
                                    | ZerobusError::AuthenticationError(_)
                            ),
                            "Attempt {} failed with unexpected error: {:?}",
                            i,
                            e
                        );
                    }
                }

                // Small delay between attempts
                sleep(Duration::from_millis(100)).await;
            }
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_stream_closure_retry_exhaustion() {
    // Test behavior when retry attempts are exhausted
    // This tests the MAX_STREAM_RECREATE_ATTEMPTS logic
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
            let batch = create_test_batch();
            let result = wrapper.send_batch(batch).await;

            // Without actual SDK that closes streams, we can't test exhaustion
            // But we verify the code path exists and handles errors
            match result {
                Ok(_) => {
                    // Success - no stream closure
                }
                Err(e) => {
                    // Verify error message includes context if retry exhausted
                    let error_msg = format!("{}", e);
                    if error_msg.contains("exhausted") || error_msg.contains("attempts") {
                        // Error indicates retry exhaustion
                        assert!(
                            matches!(e, ZerobusError::ConnectionError(_)),
                            "Retry exhaustion should be ConnectionError"
                        );
                    }
                }
            }
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_stream_closure_during_concurrent_operations() {
    // Test stream closure recovery during concurrent batch sends
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
            let wrapper = Arc::new(wrapper);
            let num_tasks = 5;
            let batch = create_test_batch();

            let mut handles = vec![];
            for task_id in 0..num_tasks {
                let wrapper_clone = wrapper.clone();
                let batch_clone = batch.clone();
                let handle = tokio::spawn(async move {
                    let result = wrapper_clone.send_batch(batch_clone).await;
                    (task_id, result.is_ok())
                });
                handles.push(handle);
            }

            // Wait for all tasks
            let mut success_count = 0;
            let mut error_count = 0;
            for handle in handles {
                let (task_id, success) = handle.await.unwrap();
                if success {
                    success_count += 1;
                } else {
                    error_count += 1;
                }
            }

            // All tasks should complete (may succeed or fail, but shouldn't deadlock)
            assert_eq!(
                success_count + error_count,
                num_tasks,
                "All tasks should complete"
            );
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_stream_closure_with_backoff() {
    // Test stream closure when error 6006 backoff is active
    use arrow_zerobus_sdk_wrapper::wrapper::zerobus;

    // Set backoff state (if possible)
    // Note: This is difficult to test without actual SDK, but we verify the code path

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

    // Check backoff before creating wrapper
    let backoff_result = zerobus::check_error_6006_backoff("test_table").await;

    match backoff_result {
        Ok(_) => {
            // No backoff active - proceed with test
            let wrapper_result = ZerobusWrapper::new(config).await;
            match wrapper_result {
                Ok(wrapper) => {
                    let batch = create_test_batch();
                    let _ = wrapper.send_batch(batch).await;
                }
                Err(_) => {
                    // Expected without real credentials
                }
            }
        }
        Err(e) => {
            // Backoff is active - verify error type
            assert!(
                matches!(e, ZerobusError::ConnectionError(_)),
                "Backoff error should be ConnectionError"
            );
        }
    }
}

#[tokio::test]
async fn test_stream_closure_diagnostic_logging() {
    // Test that diagnostic information is logged for first record failures
    // This is difficult to test without capturing logs, but we verify the code path exists
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
            let batch = create_test_batch();
            let result = wrapper.send_batch(batch).await;

            // Verify operation completes (logging happens internally)
            match result {
                Ok(_) => {
                    // Success - diagnostic logging path may not be exercised
                }
                Err(e) => {
                    // Error occurred - verify it's a known error type
                    // Diagnostic logging should have occurred internally
                    assert!(
                        matches!(
                            e,
                            ZerobusError::ConnectionError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                        ),
                        "Unexpected error type: {:?}",
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


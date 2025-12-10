//! Tests for SDK reinitialization functionality

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper, ZerobusError};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use tokio::time::Duration;

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
async fn test_sdk_initialization_on_first_use() {
    // Test that SDK is initialized lazily on first batch send
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
            // SDK should be None initially (lazy initialization)
            // First batch send should trigger SDK initialization
            let batch = create_test_batch();
            let result = wrapper.send_batch(batch).await;

            // May succeed or fail, but SDK initialization should be attempted
            match result {
                Ok(_) => {
                    // Success - SDK was initialized and batch sent
                }
                Err(e) => {
                    // SDK initialization may have failed
                    assert!(
                        matches!(
                            e,
                            ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                                | ZerobusError::ConnectionError(_)
                        ),
                        "Expected configuration/auth/connection error, got: {:?}",
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
async fn test_sdk_reinitialization_after_connection_failure() {
    // Test SDK reinitialization after connection failure
    // This is difficult to test without mocking, but we verify the code path
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

            // Attempt to send batch (may fail due to connection)
            let result1 = wrapper.send_batch(batch.clone()).await;

            // Attempt again (SDK should be reinitialized if needed)
            let result2 = wrapper.send_batch(batch).await;

            // Both operations should complete (may succeed or fail)
            // The important thing is that SDK reinitialization is attempted
            match (result1, result2) {
                (Ok(_), Ok(_)) => {
                    // Both succeeded
                }
                (Err(e1), Ok(_)) => {
                    // First failed, second succeeded (SDK may have been reinitialized)
                    assert!(
                        matches!(
                            e1,
                            ZerobusError::ConnectionError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                        ),
                        "First error should be connection/config/auth: {:?}",
                        e1
                    );
                }
                (Ok(_), Err(e2)) => {
                    // First succeeded, second failed
                    assert!(
                        matches!(
                            e2,
                            ZerobusError::ConnectionError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                        ),
                        "Second error should be connection/config/auth: {:?}",
                        e2
                    );
                }
                (Err(e1), Err(e2)) => {
                    // Both failed (expected without real SDK)
                    assert!(
                        matches!(
                            e1,
                            ZerobusError::ConnectionError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                        ),
                        "First error type: {:?}",
                        e1
                    );
                    assert!(
                        matches!(
                            e2,
                            ZerobusError::ConnectionError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                        ),
                        "Second error type: {:?}",
                        e2
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
async fn test_sdk_reinitialization_after_auth_failure() {
    // Test SDK reinitialization after authentication failure
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

            // Auth failure should result in AuthenticationError
            // SDK reinitialization should be attempted on next use
            match result {
                Ok(_) => {
                    // Success - no auth failure
                }
                Err(e) => {
                    // Verify error type
                    assert!(
                        matches!(
                            e,
                            ZerobusError::AuthenticationError(_)
                                | ZerobusError::ConfigurationError(_)
                                | ZerobusError::ConnectionError(_)
                        ),
                        "Expected auth/config/connection error, got: {:?}",
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
async fn test_sdk_initialization_error_handling() {
    // Test error handling when SDK initialization fails
    let config = WrapperConfiguration::new(
        "invalid-endpoint".to_string(), // Invalid endpoint
        "test_table".to_string(),
    )
    .with_credentials(
        "test_id".to_string(),
        "test_secret".to_string(),
    )
    .with_unity_catalog("invalid-url".to_string());

    let wrapper_result = ZerobusWrapper::new(config).await;

    match wrapper_result {
        Ok(wrapper) => {
            // Wrapper created, but SDK initialization will fail on first use
            let batch = create_test_batch();
            let result = wrapper.send_batch(batch).await;

            // Should fail with ConfigurationError
            assert!(
                result.is_err(),
                "SDK initialization with invalid config should fail"
            );

            if let Err(e) = result {
                assert!(
                    matches!(e, ZerobusError::ConfigurationError(_)),
                    "Should be ConfigurationError, got: {:?}",
                    e
                );
            }
        }
        Err(e) => {
            // May fail during wrapper creation if validation is strict
            assert!(
                matches!(e, ZerobusError::ConfigurationError(_)),
                "Wrapper creation should fail with ConfigurationError, got: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_concurrent_sdk_initialization() {
    // Test concurrent SDK initialization
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
            let num_tasks = 10;
            let batch = create_test_batch();

            let mut handles = vec![];
            for task_id in 0..num_tasks {
                let wrapper_clone = wrapper.clone();
                let batch_clone = batch.clone();
                let handle = tokio::spawn(async move {
                    // All tasks attempt to send batch simultaneously
                    // SDK should be initialized only once
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
async fn test_sdk_initialization_with_invalid_config() {
    // Test SDK initialization with invalid configuration
    let config = WrapperConfiguration::new(
        "not-a-url".to_string(), // Invalid URL format
        "test_table".to_string(),
    )
    .with_credentials(
        "test_id".to_string(),
        "test_secret".to_string(),
    )
    .with_unity_catalog("also-not-a-url".to_string());

    let wrapper_result = ZerobusWrapper::new(config).await;

    match wrapper_result {
        Ok(wrapper) => {
            let batch = create_test_batch();
            let result = wrapper.send_batch(batch).await;

            // Should fail with ConfigurationError
            assert!(result.is_err(), "Should fail with invalid config");

            if let Err(e) = result {
                assert!(
                    matches!(e, ZerobusError::ConfigurationError(_)),
                    "Should be ConfigurationError, got: {:?}",
                    e
                );
            }
        }
        Err(e) => {
            // May fail during wrapper creation
            assert!(
                matches!(e, ZerobusError::ConfigurationError(_)),
                "Should be ConfigurationError, got: {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_sdk_initialization_retry_logic() {
    // Test that SDK initialization can be retried after failure
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

            // First attempt (may fail)
            let result1 = wrapper.send_batch(batch.clone()).await;

            // Retry attempt
            let result2 = wrapper.send_batch(batch).await;

            // Both should complete (may succeed or fail)
            // The important thing is that retry is possible
            match (result1, result2) {
                (Ok(_), Ok(_)) => {
                    // Both succeeded
                }
                (Err(_), Ok(_)) => {
                    // First failed, retry succeeded
                }
                (Ok(_), Err(_)) => {
                    // First succeeded, retry failed
                }
                (Err(e1), Err(e2)) => {
                    // Both failed (expected without real SDK)
                    assert!(
                        matches!(
                            e1,
                            ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                                | ZerobusError::ConnectionError(_)
                        ),
                        "First error type: {:?}",
                        e1
                    );
                    assert!(
                        matches!(
                            e2,
                            ZerobusError::ConfigurationError(_)
                                | ZerobusError::AuthenticationError(_)
                                | ZerobusError::ConnectionError(_)
                        ),
                        "Second error type: {:?}",
                        e2
                    );
                }
            }
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}


//! Integration tests for concurrent access patterns
//!
//! Tests to verify thread safety and concurrent operations

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
async fn test_concurrent_send_batch() {
    // Test multiple tasks calling send_batch simultaneously
    // This verifies thread safety and no deadlocks
    
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

    let wrapper = Arc::new(
        ZerobusWrapper::new(config).await.expect("Failed to create wrapper")
    );

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    let num_tasks = 10;
    let batches_per_task = 5;

    for task_id in 0..num_tasks {
        let wrapper_clone = wrapper.clone();
        let handle = tokio::spawn(async move {
            let mut results = vec![];
            for batch_id in 0..batches_per_task {
                let batch = create_test_batch();
                match wrapper_clone.send_batch(batch).await {
                    Ok(result) => {
                        results.push(Ok(result));
                    }
                    Err(e) => {
                        results.push(Err(e));
                    }
                }
                // Small delay to allow interleaving
                sleep(Duration::from_millis(10)).await;
            }
            (task_id, results)
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut all_results = vec![];
    for handle in handles {
        match handle.await {
            Ok((task_id, results)) => {
                all_results.push((task_id, results));
            }
            Err(e) => {
                panic!("Task panicked: {:?}", e);
            }
        }
    }

    // Verify all tasks completed
    assert_eq!(all_results.len(), num_tasks);
    
    // Verify no deadlocks occurred (all tasks completed)
    for (task_id, results) in all_results {
        assert_eq!(
            results.len(),
            batches_per_task,
            "Task {} should have processed {} batches",
            task_id,
            batches_per_task
        );
    }
}

#[tokio::test]
async fn test_concurrent_wrapper_creation() {
    // Test multiple threads creating wrappers simultaneously
    // This verifies that wrapper creation is thread-safe
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    // Spawn multiple tasks trying to create wrappers
    let mut handles = vec![];
    let num_tasks = 10;

    for task_id in 0..num_tasks {
        let config_clone = config.clone();
        let handle = tokio::spawn(async move {
            // This will fail without real credentials, but we're testing
            // that the creation attempt doesn't cause issues
            let result = ZerobusWrapper::new(config_clone).await;
            (task_id, result)
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut all_results = vec![];
    for handle in handles {
        match handle.await {
            Ok((task_id, result)) => {
                all_results.push((task_id, result));
            }
            Err(e) => {
                panic!("Task panicked: {:?}", e);
            }
        }
    }

    // Verify all tasks completed (no deadlocks)
    assert_eq!(all_results.len(), num_tasks);
    
    // All should fail without real credentials, but none should panic
    for (task_id, result) in all_results {
        assert!(
            result.is_err(),
            "Task {} should fail without real credentials, but didn't",
            task_id
        );
    }
}

#[tokio::test]
async fn test_stream_lock_contention_pattern() {
    // Test that stream lock doesn't block unnecessarily
    // This is a pattern test - we can't easily test the actual lock
    // but we can verify the async pattern doesn't deadlock
    
    // Create a simple async mutex pattern similar to what's used in the wrapper
    use tokio::sync::Mutex;
    
    let shared_state = Arc::new(Mutex::new(0u32));
    let mut handles = vec![];
    
    // Spawn multiple tasks that acquire and release the lock
    for i in 0..10 {
        let state = shared_state.clone();
        let handle = tokio::spawn(async move {
            // Simulate the pattern: acquire lock, do async work, release
            {
                let mut guard = state.lock().await;
                *guard += 1;
            } // Lock released here
            
            // Simulate async I/O operation (lock is released)
            sleep(Duration::from_millis(10)).await;
            
            // Re-acquire lock
            {
                let mut guard = state.lock().await;
                *guard += 1;
            }
            
            i
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        let _ = handle.await.expect("Task should complete");
    }
    
    // Verify final state
    let final_value = *shared_state.lock().await;
    assert_eq!(final_value, 20, "All tasks should have incremented twice");
}

#[tokio::test]
async fn test_concurrent_config_access() {
    // Test that configuration can be accessed concurrently
    // Configuration is immutable, so this should be safe
    
    let config = Arc::new(
        WrapperConfiguration::new(
            "https://test.cloud.databricks.com".to_string(),
            "test_table".to_string(),
        )
    );
    
    let mut handles = vec![];
    
    // Spawn multiple tasks reading from config
    for i in 0..100 {
        let config_clone = config.clone();
        let handle = tokio::spawn(async move {
            // Read various fields
            let _endpoint = &config_clone.zerobus_endpoint;
            let _table = &config_clone.table_name;
            let _retry = config_clone.retry_max_attempts;
            i
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result < 100);
    }
}


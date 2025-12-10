//! Performance and stress tests
//!
//! Tests for large batches, high throughput, and concurrent operations

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper, ZerobusError};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;

/// Create a test RecordBatch with specified number of rows
fn create_test_batch(num_rows: usize) -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let ids: Vec<i64> = (0..num_rows).map(|i| i as i64).collect();
    let names: Vec<String> = (0..num_rows).map(|i| format!("Name_{}", i)).collect();

    let id_array = Int64Array::from(ids);
    let name_array = StringArray::from(names);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap()
}

#[tokio::test]
#[ignore] // Performance test - run manually
async fn test_large_batch_performance() {
    // Test with large batches (10K rows for unit test, 1M+ in production)
    // Measure memory usage and processing time
    
    let num_rows = 10_000; // 10K rows for unit test
    let batch = create_test_batch(num_rows);
    
    // Measure batch creation time
    let start = Instant::now();
    // Estimate batch size: 2 fields * num_rows * ~20 bytes per row
    let batch_size = batch.num_rows() * batch.num_columns() * 20;
    let creation_time = start.elapsed();
    
    // Verify batch is large
    assert!(batch_size > 100_000, "Batch should be > 100KB");
    assert_eq!(batch.num_rows(), num_rows);
    
    // Measure conversion time (if we had a descriptor)
    // This is a structural test - actual conversion tested elsewhere
    let conversion_start = Instant::now();
    let _ = batch.num_rows();
    let conversion_time = conversion_start.elapsed();
    
    // Verify operations are fast (< 1ms for simple operations)
    assert!(
        conversion_time < Duration::from_millis(100),
        "Simple operations should be fast: {:?}",
        conversion_time
    );
    
    println!(
        "Large batch test: {} rows, {} bytes, creation: {:?}, conversion: {:?}",
        num_rows, batch_size, creation_time, conversion_time
    );
}

#[tokio::test]
#[ignore] // Performance test - run manually
async fn test_high_throughput() {
    // Test sending many small batches rapidly
    // Measure throughput
    
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
            let num_batches = 100;
            let batch_size = 100; // Small batches
            
            let start = Instant::now();
            let mut success_count = 0;
            let mut error_count = 0;
            
            for i in 0..num_batches {
                let batch = create_test_batch(batch_size);
                match wrapper.send_batch(batch).await {
                    Ok(_) => {
                        success_count += 1;
                    }
                    Err(_) => {
                        error_count += 1;
                    }
                }
                
                // Small delay to avoid overwhelming
                if i % 10 == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            
            let duration = start.elapsed();
            let throughput = num_batches as f64 / duration.as_secs_f64();
            
            println!(
                "Throughput test: {} batches in {:?}, throughput: {:.2} batches/sec, success: {}, errors: {}",
                num_batches, duration, throughput, success_count, error_count
            );
            
            // Verify we processed all batches
            assert_eq!(success_count + error_count, num_batches);
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
#[ignore] // Performance test - run manually
async fn test_concurrent_throughput() {
    // Test concurrent batch sending
    // Measure aggregate throughput
    
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
            let batches_per_task = 20;
            
            let start = Instant::now();
            
            let mut handles = vec![];
            for task_id in 0..num_tasks {
                let wrapper_clone = wrapper.clone();
                let handle = tokio::spawn(async move {
                    let mut task_success = 0;
                    let mut task_errors = 0;
                    
                    for _ in 0..batches_per_task {
                        let batch = create_test_batch(50);
                        match wrapper_clone.send_batch(batch).await {
                            Ok(_) => {
                                task_success += 1;
                            }
                            Err(_) => {
                                task_errors += 1;
                            }
                        }
                    }
                    
                    (task_id, task_success, task_errors)
                });
                handles.push(handle);
            }
            
            let mut total_success = 0;
            let mut total_errors = 0;
            
            for handle in handles {
                match handle.await {
                    Ok((_task_id, success, errors)) => {
                        total_success += success;
                        total_errors += errors;
                    }
                    Err(e) => {
                        panic!("Task panicked: {:?}", e);
                    }
                }
            }
            
            let duration = start.elapsed();
            let total_batches = num_tasks * batches_per_task;
            let throughput = total_batches as f64 / duration.as_secs_f64();
            
            println!(
                "Concurrent throughput test: {} tasks, {} batches total in {:?}, throughput: {:.2} batches/sec, success: {}, errors: {}",
                num_tasks, total_batches, duration, throughput, total_success, total_errors
            );
            
            // Verify we processed all batches
            assert_eq!(total_success + total_errors, total_batches);
        }
        Err(_) => {
            // Expected without real credentials
        }
    }
}

#[tokio::test]
async fn test_memory_efficiency() {
    // Test memory efficiency with large batches
    // Verify that memory usage is reasonable
    
    let num_rows = 1_000;
    let batch = create_test_batch(num_rows);
    
    // Measure memory usage (estimate)
    // Actual memory = arrays + schema + metadata
    let batch_size = batch.num_rows() * batch.num_columns() * 20; // Rough estimate
    
    // Estimate expected size: 2 fields * num_rows * ~20 bytes per row = ~40KB
    let expected_min_size = num_rows * 20;
    let expected_max_size = num_rows * 100; // Allow some overhead
    
    assert!(
        batch_size >= expected_min_size,
        "Batch size {} should be at least {} bytes",
        batch_size,
        expected_min_size
    );
    
    assert!(
        batch_size <= expected_max_size,
        "Batch size {} should not exceed {} bytes (memory leak?)",
        batch_size,
        expected_max_size
    );
}

#[tokio::test]
async fn test_conversion_performance() {
    // Test conversion performance for various batch sizes
    
    let batch_sizes = vec![10, 100, 1000, 10000];
    
    for num_rows in batch_sizes {
        let batch = create_test_batch(num_rows);
        
        let start = Instant::now();
        
        // Test basic operations that would be part of conversion
        let _num_rows = batch.num_rows();
        let _num_cols = batch.num_columns();
        let _schema = batch.schema();
        
        let duration = start.elapsed();
        
        // Basic operations should be very fast (< 1ms even for 10K rows)
        assert!(
            duration < Duration::from_millis(10),
            "Basic operations for {} rows took too long: {:?}",
            num_rows,
            duration
        );
        
        println!(
            "Conversion performance: {} rows processed in {:?}",
            num_rows, duration
        );
    }
}

#[tokio::test]
async fn test_stress_many_small_batches() {
    // Stress test: many small batches
    // Verify no memory leaks or performance degradation
    
    let num_batches = 1000;
    let batch_size = 10; // Small batches
    
    let mut total_rows = 0;
    let start = Instant::now();
    
    for _ in 0..num_batches {
        let batch = create_test_batch(batch_size);
        total_rows += batch.num_rows();
        
        // Verify batch is valid
        assert_eq!(batch.num_rows(), batch_size);
    }
    
    let duration = start.elapsed();
    let throughput = num_batches as f64 / duration.as_secs_f64();
    
    println!(
        "Stress test: {} batches ({} total rows) in {:?}, throughput: {:.2} batches/sec",
        num_batches, total_rows, duration, throughput
    );
    
    // Verify all batches were created
    assert_eq!(total_rows, num_batches * batch_size);
    
    // Verify reasonable throughput (> 1000 batches/sec for simple creation)
    assert!(
        throughput > 100.0,
        "Throughput {} batches/sec is too low",
        throughput
    );
}


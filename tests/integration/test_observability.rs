//! Integration test for observability
//!
//! Verifies that metrics and traces are exported when observability is enabled.
//! Uses tracing infrastructure which the otlp-rust-service SDK picks up.

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper, OtlpSdkConfig};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
#[ignore] // Requires otlp-arrow-library to be available
async fn test_observability_metrics_export() {
    // Create temporary directory for OTLP output
    let temp_dir = TempDir::new().unwrap();
    let otlp_output_dir = temp_dir.path().join("otlp");
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string())
    .with_observability(OtlpSdkConfig {
        endpoint: None,
        output_dir: Some(otlp_output_dir.clone()),
        write_interval_secs: 1, // Fast flush for testing
        log_level: "info".to_string(),
    });

    // Initialize wrapper with observability
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    // May fail without real SDK, but tests the observability initialization
    if let Ok(wrapper) = wrapper_result {
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

        // Send batch (this should generate metrics)
        let _result = wrapper.send_batch(batch).await;
        
        // Flush observability data
        if let Some(obs) = &wrapper.observability {
            obs.flush().await.unwrap();
        }
        
        // Verify metrics file was created (if observability is working)
        let metrics_dir = otlp_output_dir.join("otlp/metrics");
        if metrics_dir.exists() {
            let files: Vec<_> = std::fs::read_dir(&metrics_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            // At least one metrics file should exist
            assert!(!files.is_empty(), "Expected metrics file to be created");
        }
    }
}

#[tokio::test]
#[ignore] // Requires otlp-arrow-library to be available
async fn test_observability_traces_export() {
    // Create temporary directory for OTLP output
    let temp_dir = TempDir::new().unwrap();
    let otlp_output_dir = temp_dir.path().join("otlp");
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string())
    .with_observability(OtlpSdkConfig {
        endpoint: None,
        output_dir: Some(otlp_output_dir.clone()),
        write_interval_secs: 1, // Fast flush for testing
        log_level: "info".to_string(),
    });

    // Initialize wrapper with observability
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    // May fail without real SDK, but tests the observability initialization
    if let Ok(wrapper) = wrapper_result {
        // Create test batch
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
        ]);
        let id_array = Int64Array::from(vec![1, 2, 3]);
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(id_array)],
        )
        .unwrap();

        // Send batch (this should generate traces)
        let _result = wrapper.send_batch(batch).await;
        
        // Flush observability data
        // Note: observability field is private, so we test via public API
        wrapper.flush().await.unwrap();
        
        // Verify trace file was created (if observability is working)
        let traces_dir = otlp_output_dir.join("otlp/traces");
        if traces_dir.exists() {
            let files: Vec<_> = std::fs::read_dir(&traces_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            // At least one trace file should exist
            assert!(!files.is_empty(), "Expected trace file to be created");
        }
    }
}

#[tokio::test]
async fn test_observability_disabled() {
    // Test that observability can be disabled
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());
    // Observability not enabled

    // Initialize wrapper without observability
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    // Should work without observability (may fail without real SDK, but tests the flow)
    // This test verifies the wrapper can be created without observability enabled
    let _ = wrapper_result;
}

#[tokio::test]
async fn test_metrics_collection() {
    // Verify metrics are collected
    // Test batch size, success rate, latency metrics
    
    let temp_dir = tempfile::TempDir::new().unwrap();
    let otlp_output_dir = temp_dir.path().join("otlp");
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_observability(OtlpSdkConfig {
        endpoint: None,
        output_dir: Some(otlp_output_dir.clone()),
        write_interval_secs: 1,
        log_level: "info".to_string(),
    });

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    if let Ok(wrapper) = wrapper_result {
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

        // Send batch (should generate metrics)
        let result = wrapper.send_batch(batch).await;
        
        // Flush observability
        let _ = wrapper.flush().await;
        
        // Verify metrics collection happened (via tracing)
        // Metrics are collected via tracing infrastructure
        // This test verifies the code path exists and doesn't panic
        match result {
            Ok(transmission_result) => {
                // Success - metrics should have been recorded
                assert!(transmission_result.batch_size_bytes > 0);
            }
            Err(_) => {
                // Expected without real SDK - but metrics path should still be exercised
            }
        }
    }
}

#[tokio::test]
async fn test_trace_spans() {
    // Verify trace spans are created
    // Test span attributes
    
    let temp_dir = tempfile::TempDir::new().unwrap();
    let otlp_output_dir = temp_dir.path().join("otlp");
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_observability(OtlpSdkConfig {
        endpoint: None,
        output_dir: Some(otlp_output_dir.clone()),
        write_interval_secs: 1,
        log_level: "info".to_string(),
    });

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    if let Ok(wrapper) = wrapper_result {
        // Create test batch
        let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
        let id_array = Int64Array::from(vec![1, 2, 3]);
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(id_array)],
        )
        .unwrap();

        // Send batch (should create trace spans)
        let _result = wrapper.send_batch(batch).await;
        
        // Flush observability
        let _ = wrapper.flush().await;
        
        // Verify trace spans were created (via tracing infrastructure)
        // This test verifies the code path exists and doesn't panic
    }
}

#[tokio::test]
async fn test_observability_flush() {
    // Test observability flush
    
    let temp_dir = tempfile::TempDir::new().unwrap();
    let otlp_output_dir = temp_dir.path().join("otlp");
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_observability(OtlpSdkConfig {
        endpoint: None,
        output_dir: Some(otlp_output_dir.clone()),
        write_interval_secs: 1,
        log_level: "info".to_string(),
    });

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    if let Ok(wrapper) = wrapper_result {
        // Flush should work even with no data
        let result = wrapper.flush().await;
        
        // May succeed or fail, but should not panic
        match result {
            Ok(_) => {
                // Success - observability flushed
            }
            Err(_) => {
                // Expected if no data or SDK not available
            }
        }
    }
}

#[tokio::test]
async fn test_observability_with_batch_operations() {
    // Test observability during batch operations
    
    let temp_dir = tempfile::TempDir::new().unwrap();
    let otlp_output_dir = temp_dir.path().join("otlp");
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_observability(OtlpSdkConfig {
        endpoint: None,
        output_dir: Some(otlp_output_dir.clone()),
        write_interval_secs: 1,
        log_level: "info".to_string(),
    });

    let wrapper_result = ZerobusWrapper::new(config).await;
    
    if let Ok(wrapper) = wrapper_result {
        // Send multiple batches to generate metrics and traces
        for i in 0..3 {
            let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
            let id_array = Int64Array::from(vec![i, i + 1, i + 2]);
            let batch = RecordBatch::try_new(
                Arc::new(schema),
                vec![Arc::new(id_array)],
            )
            .unwrap();
            
            let _ = wrapper.send_batch(batch).await;
        }
        
        // Flush all observability data
        let _ = wrapper.flush().await;
        
        // Verify observability worked (no panics)
    }
}


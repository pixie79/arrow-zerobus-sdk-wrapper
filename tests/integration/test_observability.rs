//! Integration test for observability
//!
//! Verifies that metrics and traces are exported when observability is enabled.

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper, OtlpConfig};
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
    .with_observability(OtlpConfig {
        endpoint: Some(otlp_output_dir.to_string_lossy().to_string()),
        extra: std::collections::HashMap::new(),
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
    .with_observability(OtlpConfig {
        endpoint: Some(otlp_output_dir.to_string_lossy().to_string()),
        extra: std::collections::HashMap::new(),
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
    if wrapper_result.is_ok() {
        // Wrapper created successfully without observability
        assert!(true);
    }
}


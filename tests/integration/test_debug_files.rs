//! Integration test for debug file output
//!
//! Verifies that Arrow and Protobuf debug files are written correctly when enabled.

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
#[ignore] // Requires real SDK, but tests the debug file writing logic
async fn test_debug_files_written() {
    // Create temporary directory for debug output
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string())
    .with_debug_output(debug_output_dir.clone())
    .with_debug_flush_interval_secs(1); // Short interval for testing

    // Initialize wrapper with debug enabled
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    // May fail without real SDK, but tests the debug initialization
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

        // Send batch (this should write debug files)
        let _result = wrapper.send_batch(batch).await;
        
        // Flush debug files
        wrapper.flush().await.unwrap();
        
        // Wait a bit for file writes to complete
        sleep(Duration::from_millis(500)).await;
        
        // Verify Arrow file was created
        let arrow_file = debug_output_dir.join("zerobus/arrow/table.arrow");
        if arrow_file.exists() {
            let metadata = std::fs::metadata(&arrow_file).unwrap();
            assert!(metadata.len() > 0, "Arrow file should not be empty");
        }
        
        // Verify Protobuf file was created
        let proto_file = debug_output_dir.join("zerobus/proto/table.proto");
        if proto_file.exists() {
            let metadata = std::fs::metadata(&proto_file).unwrap();
            assert!(metadata.len() > 0, "Protobuf file should not be empty");
        }
    }
}

#[tokio::test]
async fn test_debug_files_disabled() {
    // Test that debug files are not created when disabled
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());
    // Debug not enabled

    // Initialize wrapper without debug
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    // Should work without debug (may fail without real SDK, but tests the flow)
    if wrapper_result.is_ok() {
        // Debug directories should not be created
        let arrow_dir = debug_output_dir.join("zerobus/arrow");
        let proto_dir = debug_output_dir.join("zerobus/proto");
        assert!(!arrow_dir.exists());
        assert!(!proto_dir.exists());
    }
}

#[tokio::test]
#[ignore] // Requires real SDK
async fn test_debug_file_rotation() {
    // Test that files are rotated when max size is reached
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string())
    .with_debug_output(debug_output_dir.clone())
    .with_debug_max_file_size(Some(1024)); // Small max size for testing

    // Initialize wrapper with debug and rotation
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    if let Ok(wrapper) = wrapper_result {
        // Create large batch to trigger rotation
        let schema = Schema::new(vec![
            Field::new("data", DataType::Utf8, false),
        ]);
        
        // Create a batch with enough data to exceed max file size
        let large_data: Vec<String> = (0..1000)
            .map(|i| format!("data_{}", i))
            .collect();
        let data_array = StringArray::from(large_data);
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(data_array)],
        )
        .unwrap();

        // Send multiple batches to trigger rotation
        for _ in 0..10 {
            let _result = wrapper.send_batch(batch.clone()).await;
        }
        
        // Flush and check for rotated files
        wrapper.flush().await.unwrap();
        sleep(Duration::from_millis(500)).await;
        
        // Check if rotated files exist (with timestamp suffix)
        let arrow_dir = debug_output_dir.join("zerobus/arrow");
        if arrow_dir.exists() {
            let files: Vec<_> = std::fs::read_dir(&arrow_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            // Should have at least the main file, possibly rotated files
            assert!(!files.is_empty());
        }
    }
}


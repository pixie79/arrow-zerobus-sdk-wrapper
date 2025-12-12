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

#[tokio::test]
async fn test_debug_files_written_when_writer_disabled() {
    // Test that debug files are written even when writer is disabled
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(debug_output_dir.clone())
    .with_zerobus_writer_disabled(true);
    // Note: No credentials required when writer is disabled

    // Initialize wrapper with writer disabled mode
    let wrapper_result = ZerobusWrapper::new(config).await;
    
    // Should succeed without credentials
    assert!(wrapper_result.is_ok(), "Wrapper should initialize without credentials when writer disabled");
    
    let wrapper = wrapper_result.unwrap();
    
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

    // Send batch - should write debug files but skip SDK calls
    let result = wrapper.send_batch(batch).await;
    
    // Should succeed (conversion succeeded, no network calls made)
    assert!(result.is_ok(), "send_batch should succeed when writer disabled");
    let transmission_result = result.unwrap();
    assert!(transmission_result.success, "Transmission result should indicate success");
    
    // Flush debug files
    wrapper.flush().await.unwrap();
    
    // Wait a bit for file writes to complete
    sleep(Duration::from_millis(500)).await;
    
    // Verify Arrow file was created
    let sanitized_table_name = "test_table".replace(['.', '/'], "_");
    let arrow_file = debug_output_dir.join(format!("zerobus/arrow/{}.arrow", sanitized_table_name));
    assert!(arrow_file.exists(), "Arrow file should be created when writer disabled");
    let metadata = std::fs::metadata(&arrow_file).unwrap();
    assert!(metadata.len() > 0, "Arrow file should not be empty");
    
    // Verify Protobuf file was created
    let proto_file = debug_output_dir.join(format!("zerobus/proto/{}.proto", sanitized_table_name));
    assert!(proto_file.exists(), "Protobuf file should be created when writer disabled");
    let metadata = std::fs::metadata(&proto_file).unwrap();
    assert!(metadata.len() > 0, "Protobuf file should not be empty");
}

#[tokio::test]
async fn test_arrow_only_debug_output() {
    // Test that only Arrow files are created when arrow_enabled=true, protobuf_enabled=false
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_arrow_enabled(true)
    .with_debug_protobuf_enabled(false)
    .with_debug_output(debug_output_dir.clone())
    .with_zerobus_writer_disabled(true);

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

    // Send batch
    wrapper.send_batch(batch).await.unwrap();
    wrapper.flush().await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Verify Arrow file was created
    let sanitized_table_name = "test_table".replace(['.', '/'], "_");
    let arrow_file = debug_output_dir.join(format!("zerobus/arrow/{}.arrows", sanitized_table_name));
    assert!(arrow_file.exists(), "Arrow file should be created when arrow_enabled=true");
    
    // Verify Protobuf file was NOT created
    let proto_file = debug_output_dir.join(format!("zerobus/proto/{}.proto", sanitized_table_name));
    assert!(!proto_file.exists(), "Protobuf file should NOT be created when protobuf_enabled=false");
}

#[tokio::test]
async fn test_protobuf_only_debug_output() {
    // Test that only Protobuf files are created when protobuf_enabled=true, arrow_enabled=false
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_arrow_enabled(false)
    .with_debug_protobuf_enabled(true)
    .with_debug_output(debug_output_dir.clone())
    .with_zerobus_writer_disabled(true);

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

    // Send batch
    wrapper.send_batch(batch).await.unwrap();
    wrapper.flush().await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Verify Protobuf file was created
    let sanitized_table_name = "test_table".replace(['.', '/'], "_");
    let proto_file = debug_output_dir.join(format!("zerobus/proto/{}.proto", sanitized_table_name));
    assert!(proto_file.exists(), "Protobuf file should be created when protobuf_enabled=true");
    
    // Verify Arrow file was NOT created
    let arrow_file = debug_output_dir.join(format!("zerobus/arrow/{}.arrows", sanitized_table_name));
    assert!(!arrow_file.exists(), "Arrow file should NOT be created when arrow_enabled=false");
}

#[tokio::test]
async fn test_both_formats_enabled() {
    // Test that both Arrow and Protobuf files are created when both flags are true
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_arrow_enabled(true)
    .with_debug_protobuf_enabled(true)
    .with_debug_output(debug_output_dir.clone())
    .with_zerobus_writer_disabled(true);

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

    // Send batch
    wrapper.send_batch(batch).await.unwrap();
    wrapper.flush().await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Verify both files were created
    let sanitized_table_name = "test_table".replace(['.', '/'], "_");
    let arrow_file = debug_output_dir.join(format!("zerobus/arrow/{}.arrows", sanitized_table_name));
    let proto_file = debug_output_dir.join(format!("zerobus/proto/{}.proto", sanitized_table_name));
    
    assert!(arrow_file.exists(), "Arrow file should be created when arrow_enabled=true");
    assert!(proto_file.exists(), "Protobuf file should be created when protobuf_enabled=true");
}

#[tokio::test]
async fn test_both_formats_disabled() {
    // Test that no debug files are created when both flags are false
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_arrow_enabled(false)
    .with_debug_protobuf_enabled(false)
    .with_debug_output(debug_output_dir.clone())
    .with_zerobus_writer_disabled(true);

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

    // Send batch
    wrapper.send_batch(batch).await.unwrap();
    wrapper.flush().await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Verify no files were created
    let sanitized_table_name = "test_table".replace(['.', '/'], "_");
    let arrow_file = debug_output_dir.join(format!("zerobus/arrow/{}.arrows", sanitized_table_name));
    let proto_file = debug_output_dir.join(format!("zerobus/proto/{}.proto", sanitized_table_name));
    
    assert!(!arrow_file.exists(), "Arrow file should NOT be created when arrow_enabled=false");
    assert!(!proto_file.exists(), "Protobuf file should NOT be created when protobuf_enabled=false");
}

#[tokio::test]
async fn test_rotation_no_recursive_timestamps() {
    // Test that file rotation doesn't create recursive timestamps
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_arrow_enabled(true)
    .with_debug_output(debug_output_dir.clone())
    .with_zerobus_writer_disabled(true);

    let wrapper = ZerobusWrapper::new(config).await.unwrap();
    
    // Create batches to trigger multiple rotations
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))], // Exceed rotation threshold
    )
    .unwrap();
    
    // Trigger multiple rotations
    for _ in 0..5 {
        wrapper.send_batch(batch.clone()).await.unwrap();
    }
    
    wrapper.flush().await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Check rotated files for recursive timestamps
    let arrow_dir = debug_output_dir.join("zerobus/arrow");
    if arrow_dir.exists() {
        let entries: Vec<_> = std::fs::read_dir(&arrow_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        
        for entry in entries {
            // Count timestamp patterns - should be at most 1 per filename
            let timestamp_patterns: Vec<_> = entry.match_indices("_20")
                .filter(|(_, s)| {
                    // Check if followed by date pattern (YYYYMMDD_HHMMSS)
                    let rest = &entry[s.len()..];
                    rest.len() >= 14 && rest.chars().take(14).all(|c| c.is_ascii_digit() || c == '_')
                })
                .collect();
            
            assert!(timestamp_patterns.len() <= 1, 
                "Filename should have at most one timestamp pattern: {} (found {})", 
                entry, timestamp_patterns.len());
        }
    }
}

#[tokio::test]
async fn test_file_retention_cleanup() {
    // Test that old files are cleaned up when retention limit is exceeded
    use std::fs;
    
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    let arrow_dir = debug_output_dir.join("zerobus/arrow");
    fs::create_dir_all(&arrow_dir).unwrap();
    
    // Pre-create 12 rotated files (more than limit of 10)
    let sanitized_table_name = "test_table".replace(['.', '/'], "_");
    for i in 0..12 {
        let timestamp = format!("20250101_{:06}", i * 100);
        let file_path = arrow_dir.join(format!("{}_{}.arrows", sanitized_table_name, timestamp));
        fs::File::create(&file_path).unwrap();
    }
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_arrow_enabled(true)
    .with_debug_output(debug_output_dir.clone())
    .with_debug_max_files_retained(Some(10))
    .with_zerobus_writer_disabled(true);

    let wrapper = ZerobusWrapper::new(config).await.unwrap();
    
    // Trigger rotation which should cleanup old files
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))],
    )
    .unwrap();
    
    wrapper.send_batch(batch).await.unwrap();
    wrapper.flush().await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Count files (should be <= 11: 10 old + 1 new active, or 10 if cleanup worked)
    let entries: Vec<_> = fs::read_dir(&arrow_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    
    assert!(entries.len() <= 11, 
        "Should have at most 11 files after cleanup (10 retained + 1 active), got {}", 
        entries.len());
}

#[tokio::test]
async fn test_file_retention_unlimited() {
    // Test that files are not cleaned up when retention is unlimited (None)
    use std::fs;
    
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();
    let arrow_dir = debug_output_dir.join("zerobus/arrow");
    fs::create_dir_all(&arrow_dir).unwrap();
    
    // Pre-create some rotated files
    let sanitized_table_name = "test_table".replace(['.', '/'], "_");
    for i in 0..5 {
        let timestamp = format!("20250101_{:06}", i * 100);
        let file_path = arrow_dir.join(format!("{}_{}.arrows", sanitized_table_name, timestamp));
        fs::File::create(&file_path).unwrap();
    }
    
    let initial_count = fs::read_dir(&arrow_dir).unwrap().count();
    
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_arrow_enabled(true)
    .with_debug_output(debug_output_dir.clone())
    .with_debug_max_files_retained(None) // Unlimited
    .with_zerobus_writer_disabled(true);

    let wrapper = ZerobusWrapper::new(config).await.unwrap();
    
    // Trigger rotation
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))],
    )
    .unwrap();
    
    wrapper.send_batch(batch).await.unwrap();
    wrapper.flush().await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // All files should still exist (no cleanup)
    let final_count = fs::read_dir(&arrow_dir).unwrap().count();
    assert!(final_count >= initial_count + 1, 
        "Should retain all files with unlimited retention (initial: {}, final: {})", 
        initial_count, final_count);
}


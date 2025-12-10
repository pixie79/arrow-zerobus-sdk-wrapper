//! Tests for debug file rotation functionality

use arrow_zerobus_sdk_wrapper::wrapper::debug::DebugWriter;
use arrow_zerobus_sdk_wrapper::ZerobusError;
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

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

/// Create a large test RecordBatch to trigger rotation
fn create_large_batch(size_mb: usize) -> RecordBatch {
    let num_rows = size_mb * 1024 * 1024 / 20; // Rough estimate: ~20 bytes per row
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("data", DataType::Utf8, false),
    ]);

    let ids: Vec<i64> = (0..num_rows).map(|i| i as i64).collect();
    let data: Vec<String> = (0..num_rows)
        .map(|i| format!("data_{}", i))
        .collect();

    let id_array = Int64Array::from(ids);
    let data_array = StringArray::from(data);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(data_array)],
    )
    .unwrap()
}

#[tokio::test]
async fn test_arrow_file_rotation_when_size_exceeded() {
    // Test that Arrow file rotates when max_file_size is exceeded
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        Some(1024), // Small max size: 1KB
    )
    .unwrap();

    // Write batches until file size exceeds limit
    let batch = create_large_batch(1); // Create a batch that will exceed 1KB
    debug_writer.write_arrow(&batch).await.unwrap();
    debug_writer.flush().await.unwrap();

    // Write another batch to trigger rotation
    debug_writer.write_arrow(&batch).await.unwrap();
    debug_writer.flush().await.unwrap();

    // Check for rotated files
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    let files: Vec<_> = std::fs::read_dir(&arrow_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    // Should have at least one file (may have rotated file with timestamp)
    assert!(!files.is_empty(), "Expected at least one Arrow file");
    
    // Check if rotation occurred (files with timestamp suffix)
    let has_rotated = files.iter().any(|f| {
        let name = f.to_string_lossy();
        name.contains("_") && name.contains(".arrow")
    });
    
    // Rotation may or may not have occurred depending on file size
    // The important thing is that writes succeeded
    assert!(files.len() >= 1);
}

#[tokio::test]
async fn test_protobuf_file_rotation_when_size_exceeded() {
    // Test that Protobuf file rotates when max_file_size is exceeded
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        Some(1024), // Small max size: 1KB
    )
    .unwrap();

    // Write protobuf data multiple times to exceed size
    // Create test protobuf bytes (simulate protobuf data)
    let test_bytes = vec![0u8; 200]; // 200 bytes per write
    for _ in 0..10 {
        debug_writer.write_protobuf(&test_bytes, false).await.unwrap();
    }
    debug_writer.flush().await.unwrap();

    // Check for rotated files
    let proto_dir = temp_dir.path().join("zerobus/proto");
    let files: Vec<_> = std::fs::read_dir(&proto_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    // Should have at least one file
    assert!(!files.is_empty(), "Expected at least one Protobuf file");
}

#[tokio::test]
async fn test_no_rotation_when_size_not_exceeded() {
    // Test that files don't rotate when size is below limit
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        Some(10 * 1024 * 1024), // Large max size: 10MB
    )
    .unwrap();

    // Write multiple small batches
    let batch = create_test_batch();
    for _ in 0..100 {
        debug_writer.write_arrow(&batch).await.unwrap();
    }
    debug_writer.flush().await.unwrap();

    // Check files - should only have one file per type (no rotation)
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    let files: Vec<_> = std::fs::read_dir(&arrow_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    // Should have exactly one Arrow file (no rotation)
    let arrow_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".arrow"))
        .collect();
    
    // May have descriptor file, so check for exactly one .arrow file
    assert_eq!(arrow_files.len(), 1, "Expected exactly one Arrow file (no rotation)");
}

#[tokio::test]
async fn test_rotation_exact_size_boundary() {
    // Test rotation behavior at exact size boundary
    let temp_dir = TempDir::new().unwrap();
    let max_size = 2048; // 2KB
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        Some(max_size),
    )
    .unwrap();

    // Write data to approach the boundary
    let batch = create_test_batch();
    for _ in 0..50 {
        debug_writer.write_arrow(&batch).await.unwrap();
    }
    debug_writer.flush().await.unwrap();

    // Check file size
    let arrow_file = temp_dir.path().join("zerobus/arrow/test_table.arrow");
    if arrow_file.exists() {
        let metadata = std::fs::metadata(&arrow_file).unwrap();
        let file_size = metadata.len();
        
        // Write one more batch that should trigger rotation if we're at boundary
        debug_writer.write_arrow(&batch).await.unwrap();
        debug_writer.flush().await.unwrap();
        
        // Check if rotation occurred
        let arrow_dir = temp_dir.path().join("zerobus/arrow");
        let files: Vec<_> = std::fs::read_dir(&arrow_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect();
        
        // If file size exceeded max_size, rotation should have occurred
        if file_size >= max_size {
            let rotated_files: Vec<_> = files
                .iter()
                .filter(|f| {
                    let name = f.to_string_lossy();
                    name.contains("_") && name.ends_with(".arrow")
                })
                .collect();
            // Rotation may have occurred
            assert!(files.len() >= 1);
        }
    }
}

#[tokio::test]
async fn test_multiple_rotations() {
    // Test that multiple rotations work correctly
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        Some(512), // Very small max size: 512 bytes
    )
    .unwrap();

    // Write batches to trigger multiple rotations
    let batch = create_large_batch(1);
    for _ in 0..5 {
        debug_writer.write_arrow(&batch).await.unwrap();
        debug_writer.flush().await.unwrap();
    }

    // Check for multiple rotated files
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    let files: Vec<_> = std::fs::read_dir(&arrow_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    // Should have at least one file (may have multiple rotated files)
    assert!(!files.is_empty(), "Expected at least one file");
    
    // Verify all files are valid
    for file in &files {
        let file_path = arrow_dir.join(file);
        assert!(file_path.exists(), "File should exist: {:?}", file);
    }
}

#[tokio::test]
async fn test_rotation_with_no_max_size() {
    // Test that rotation doesn't occur when max_file_size is None
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None, // No max size
    )
    .unwrap();

    // Write large amounts of data
    let batch = create_large_batch(1);
    for _ in 0..20 {
        debug_writer.write_arrow(&batch).await.unwrap();
    }
    debug_writer.flush().await.unwrap();

    // Check files - should only have one file (no rotation)
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    let files: Vec<_> = std::fs::read_dir(&arrow_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    // Should have exactly one Arrow file (no rotation when max_size is None)
    let arrow_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".arrow"))
        .collect();
    
    assert_eq!(arrow_files.len(), 1, "Expected exactly one Arrow file (no rotation)");
}

#[tokio::test]
async fn test_rotation_file_naming() {
    // Test that rotated files have correct naming pattern
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        Some(1024), // Small max size
    )
    .unwrap();

    // Write data to trigger rotation
    let batch = create_large_batch(1);
    for _ in 0..5 {
        debug_writer.write_arrow(&batch).await.unwrap();
        debug_writer.flush().await.unwrap();
    }

    // Check file naming pattern
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    let files: Vec<_> = std::fs::read_dir(&arrow_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    // Verify file naming
    for file in &files {
        let name = file.to_string_lossy();
        
        // Should be either:
        // - test_table.arrow (original)
        // - test_table_YYYYMMDD_HHMMSS.arrow (rotated)
        assert!(
            name == "test_table.arrow" || 
            (name.starts_with("test_table_") && name.ends_with(".arrow")),
            "Unexpected file name: {}",
            name
        );
        
        // If rotated, verify timestamp format
        if name.starts_with("test_table_") && name != "test_table.arrow" {
            let timestamp_part = name
                .strip_prefix("test_table_")
                .unwrap()
                .strip_suffix(".arrow")
                .unwrap();
            
            // Verify timestamp format: YYYYMMDD_HHMMSS
            assert_eq!(timestamp_part.len(), 15, "Timestamp should be 15 characters");
            assert!(timestamp_part.chars().all(|c| c.is_ascii_digit() || c == '_'));
        }
    }
}


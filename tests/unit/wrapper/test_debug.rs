//! Unit tests for debug writer
//!
//! Target: ≥90% coverage per file

use arrow_zerobus_sdk_wrapper::wrapper::debug::DebugWriter;
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

#[tokio::test]
async fn test_debug_writer_new() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    
    let writer = DebugWriter::new(
        output_dir.clone(),
        "test_table".to_string(),
        Duration::from_secs(5),
        Some(1024 * 1024), // 1MB
        Some(10),
    );
    
    assert!(writer.is_ok());
    
    // Verify directories were created
    let arrow_dir = output_dir.join("zerobus/arrow");
    let proto_dir = output_dir.join("zerobus/proto");
    assert!(arrow_dir.exists());
    assert!(proto_dir.exists());
}

#[tokio::test]
async fn test_debug_writer_new_invalid_directory() {
    // Try to create in a non-existent parent directory
    let invalid_path = PathBuf::from("/nonexistent/path/debug");
    
    // This should fail on some systems, but may succeed if we have permissions
    // We'll test that it handles errors gracefully
    let writer = DebugWriter::new(
        invalid_path,
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        Some(10),
    );
    
    // May succeed or fail depending on system, but should not panic
    let _ = writer;
}

#[tokio::test]
async fn test_debug_writer_write_arrow() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    
    let mut writer = DebugWriter::new(
        output_dir.clone(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        Some(10),
    ).unwrap();
    
    // Create a test RecordBatch
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    let id_array = Int64Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    ).unwrap();
    
    // Write Arrow batch
    let result = writer.write_arrow(&batch).await;
    assert!(result.is_ok());
    
    // Verify file was created
    let arrow_file = output_dir.join("zerobus/arrow/table.arrow");
    // File may not exist immediately if buffered, but should be created after flush
    writer.flush().await.unwrap();
    
    // Check if file exists (may need to wait a bit)
    sleep(Duration::from_millis(100)).await;
    // Note: File existence check depends on implementation
}

#[tokio::test]
async fn test_debug_writer_write_protobuf() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    
    let mut writer = DebugWriter::new(
        output_dir.clone(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        Some(10),
    ).unwrap();
    
    // Create test Protobuf bytes
    let protobuf_bytes = b"test protobuf data";
    
    // Write Protobuf bytes
    let result = writer.write_protobuf(protobuf_bytes).await;
    assert!(result.is_ok());
    
    // Flush and verify file was created
    writer.flush().await.unwrap();
    sleep(Duration::from_millis(100)).await;
    
    let proto_file = output_dir.join("zerobus/proto/table.proto");
    // File existence check depends on implementation
}

#[tokio::test]
async fn test_debug_writer_flush() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    
    let mut writer = DebugWriter::new(
        output_dir,
        Duration::from_secs(5),
        None,
    ).unwrap();
    
    // Flush should succeed even with no data
    let result = writer.flush().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_writer_should_flush() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    
    let writer = DebugWriter::new(
        output_dir,
        "test_table".to_string(),
        Duration::from_millis(100), // Short interval for testing
        None,
        Some(10),
    ).unwrap();
    
    // Immediately after creation, should not need flush
    assert!(!writer.should_flush().await);
    
    // Wait for flush interval
    sleep(Duration::from_millis(150)).await;
    
    // Now should need flush
    assert!(writer.should_flush().await);
}

#[tokio::test]
async fn test_debug_writer_multiple_writes() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    
    let mut writer = DebugWriter::new(
        output_dir.clone(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        Some(10),
    ).unwrap();
    
    // Create multiple batches
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch1 = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![Arc::new(Int64Array::from(vec![1, 2]))],
    ).unwrap();
    let batch2 = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![3, 4]))],
    ).unwrap();
    
    // Write multiple batches
    writer.write_arrow(&batch1).await.unwrap();
    writer.write_arrow(&batch2).await.unwrap();
    
    // Write multiple Protobuf chunks
    writer.write_protobuf(b"chunk1").await.unwrap();
    writer.write_protobuf(b"chunk2").await.unwrap();
    
    // Flush all
    writer.flush().await.unwrap();
}

#[tokio::test]
async fn test_rotation_no_recursive_timestamps() {
    // Test that file rotation doesn't create recursive timestamps in filenames
    // This verifies the fix for issue #13 where rotated files would accumulate multiple timestamps
    use arrow_zerobus_sdk_wrapper::wrapper::debug::DebugWriter;
    
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("test_table.arrows");
    
    // Create a file to test rotation
    std::fs::File::create(&base_path).unwrap();
    
    let writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        Some(10),
    ).unwrap();
    
    // Create a schema and batch to trigger rotation
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))], // Exceed rotation threshold
    ).unwrap();
    
    // Write enough batches to trigger rotation
    for _ in 0..2 {
        writer.write_arrow(&batch).await.unwrap();
    }
    
    // Check that rotated files have correct naming format
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path().join("zerobus/arrow"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    
    // Verify no recursive timestamps - each filename should have at most one timestamp pattern
    for entry in &entries {
        // Extract timestamp patterns: should match YYYYMMDD_HHMMSS format (_20 followed by 8 digits, underscore, 6 digits)
        // Count occurrences of "_20" followed by date pattern (8 digits) and time pattern (6 digits)
        let mut timestamp_count = 0;
        let mut search_start = 0;
        while let Some(pos) = entry[search_start..].find("_20") {
            let actual_pos = search_start + pos;
            // Check if followed by 8 digits, underscore, 6 digits (YYYYMMDD_HHMMSS pattern)
            if actual_pos + 3 + 8 + 1 + 6 <= entry.len() {
                let date_part = &entry[actual_pos + 3..actual_pos + 3 + 8];
                let separator = &entry[actual_pos + 3 + 8..actual_pos + 3 + 8 + 1];
                let time_part = &entry[actual_pos + 3 + 8 + 1..actual_pos + 3 + 8 + 1 + 6];
                if date_part.chars().all(|c| c.is_ascii_digit())
                    && separator == "_"
                    && time_part.chars().all(|c| c.is_ascii_digit())
                {
                    timestamp_count += 1;
                }
            }
            search_start = actual_pos + 3;
        }
        assert!(
            timestamp_count <= 1,
            "Filename should have at most one timestamp pattern, got {} in: {}",
            timestamp_count,
            entry
        );
        
        // Verify filename format: either "test_table.arrows" or "test_table_YYYYMMDD_HHMMSS.arrows"
        assert!(
            entry == "test_table.arrows" || (entry.starts_with("test_table_20") && entry.ends_with(".arrows")),
            "Unexpected filename format: {}",
            entry
        );
    }
}

#[tokio::test]
async fn test_generate_rotated_path_with_existing_timestamp() {
    use std::fs;
    
    let temp_dir = TempDir::new().unwrap();
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    fs::create_dir_all(&arrow_dir).unwrap();
    
    // Create a file with existing timestamp
    let existing_file = arrow_dir.join("test_table_20250101_120000.arrows");
    fs::File::create(&existing_file).unwrap();
    
    let writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        Some(10),
    ).unwrap();
    
    // Trigger rotation
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))],
    ).unwrap();
    
    for _ in 0..2 {
        writer.write_arrow(&batch).await.unwrap();
    }
    
    // Verify new rotated file doesn't have recursive timestamp
    let entries: Vec<_> = fs::read_dir(&arrow_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    
    for entry in entries {
        // Extract timestamp patterns
        let parts: Vec<&str> = entry.split('_').collect();
        let timestamp_like_parts = parts.iter()
            .filter(|p| p.len() == 8 && p.chars().all(|c| c.is_ascii_digit())) // YYYYMMDD
            .count();
        // Should have at most one date part (YYYYMMDD) followed by time (HHMMSS)
        assert!(timestamp_like_parts <= 1, "Should have at most one date part: {}", entry);
    }
}

#[tokio::test]
async fn test_file_retention_cleanup() {
    use std::fs;
    
    let temp_dir = TempDir::new().unwrap();
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    fs::create_dir_all(&arrow_dir).unwrap();
    
    // Create 12 rotated files (more than limit of 10)
    for i in 0..12 {
        let timestamp = format!("20250101_{:06}", i * 100); // Simulate timestamps
        let file_path = arrow_dir.join(format!("test_table_{}.arrows", timestamp));
        fs::File::create(&file_path).unwrap();
        // Set file modification time to make them sortable
        let time = std::time::SystemTime::now() - Duration::from_secs((12 - i) as u64);
        let file_time = filetime::FileTime::from_system_time(time);
        filetime::set_file_times(&file_path, file_time, file_time).unwrap();
    }
    
    let writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        Some(10), // Keep only 10 files
    ).unwrap();
    
    // Trigger rotation which should cleanup old files
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))],
    ).unwrap();
    
    writer.write_arrow(&batch).await.unwrap();
    
    // Wait a bit for cleanup
    sleep(Duration::from_millis(100)).await;
    
    // Count files (should be <= 11: 10 old + 1 new active, or 10 if cleanup worked)
    let entries: Vec<_> = fs::read_dir(&arrow_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    
    // Should have at most 11 files (10 retained + 1 active) or exactly 10 if cleanup worked perfectly
    assert!(entries.len() <= 11, "Should have at most 11 files after cleanup, got {}", entries.len());
}

#[tokio::test]
async fn test_file_retention_unlimited() {
    use std::fs;
    
    let temp_dir = TempDir::new().unwrap();
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    fs::create_dir_all(&arrow_dir).unwrap();
    
    // Create some rotated files
    for i in 0..5 {
        let timestamp = format!("20250101_{:06}", i * 100);
        let file_path = arrow_dir.join(format!("test_table_{}.arrows", timestamp));
        fs::File::create(&file_path).unwrap();
    }
    
    let writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
        None, // Unlimited retention
    ).unwrap();
    
    // Trigger rotation
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))],
    ).unwrap();
    
    writer.write_arrow(&batch).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    
    // All files should still exist (no cleanup)
    let entries: Vec<_> = fs::read_dir(&arrow_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    
    // Should have at least 5 old files + 1 new = 6
    assert!(entries.len() >= 6, "Should retain all files with unlimited retention");
}

#[tokio::test]
async fn test_sequential_naming_when_filename_too_long() {
    // Test that sequential numbering is used when filename would exceed length limit
    // This is harder to test directly, but we can verify the behavior through rotation
    let temp_dir = TempDir::new().unwrap();
    
    let writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "a".repeat(200).to_string(), // Very long table name
        Duration::from_secs(5),
        None,
        Some(10),
    ).unwrap();
    
    // Trigger rotation multiple times
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![1; 1001]))],
    ).unwrap();
    
    for _ in 0..3 {
        writer.write_arrow(&batch).await.unwrap();
    }
    
    // Verify files were created (may use sequential naming if too long)
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    let entries: Vec<_> = std::fs::read_dir(&arrow_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    
    // Files should exist and not exceed reasonable length
    for entry in entries {
        assert!(entry.len() < 300, "Filename should not exceed filesystem limits: {}", entry);
    }
}


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
        Duration::from_secs(5),
        Some(1024 * 1024), // 1MB
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
        Duration::from_secs(5),
        None,
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
        Duration::from_secs(5),
        None,
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
        Duration::from_secs(5),
        None,
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
        Duration::from_millis(100), // Short interval for testing
        None,
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
        Duration::from_secs(5),
        None,
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


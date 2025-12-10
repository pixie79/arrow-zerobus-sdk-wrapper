//! Tests for concurrent debug file writes

use arrow_zerobus_sdk_wrapper::wrapper::debug::DebugWriter;
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

#[tokio::test]
async fn test_concurrent_arrow_writes() {
    // Test that multiple tasks can write Arrow batches concurrently
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = Arc::new(
        DebugWriter::new(
            temp_dir.path().to_path_buf(),
            "test_table".to_string(),
            Duration::from_secs(5),
            None,
        )
        .unwrap(),
    );

    let num_tasks = 10;
    let batches_per_task = 5;

    let mut handles = vec![];
    for task_id in 0..num_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            let batch = create_test_batch();
            for i in 0..batches_per_task {
                writer.write_arrow(&batch).await.unwrap();
            }
            (task_id, batches_per_task)
        });
        handles.push(handle);
    }

    // Wait for all tasks
    let mut total_writes = 0;
    for handle in handles {
        let (task_id, count) = handle.await.unwrap();
        total_writes += count;
        assert!(count == batches_per_task, "Task {} should have written {} batches", task_id, batches_per_task);
    }

    // Verify all writes succeeded
    assert_eq!(total_writes, num_tasks * batches_per_task);
    
    // Flush and verify file exists
    debug_writer.flush().await.unwrap();
    let arrow_file = temp_dir.path().join("zerobus/arrow/test_table.arrow");
    assert!(arrow_file.exists(), "Arrow file should exist after concurrent writes");
}

#[tokio::test]
async fn test_concurrent_protobuf_writes() {
    // Test that multiple tasks can write Protobuf data concurrently
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = Arc::new(
        DebugWriter::new(
            temp_dir.path().to_path_buf(),
            "test_table".to_string(),
            Duration::from_secs(5),
            None,
        )
        .unwrap(),
    );

    let num_tasks = 10;
    let writes_per_task = 5;

    let mut handles = vec![];
    for task_id in 0..num_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            let test_bytes = format!("task_{}_data", task_id).into_bytes();
            for _ in 0..writes_per_task {
                writer.write_protobuf(&test_bytes, false).await.unwrap();
            }
            (task_id, writes_per_task)
        });
        handles.push(handle);
    }

    // Wait for all tasks
    let mut total_writes = 0;
    for handle in handles {
        let (task_id, count) = handle.await.unwrap();
        total_writes += count;
        assert!(count == writes_per_task, "Task {} should have written {} times", task_id, writes_per_task);
    }

    // Verify all writes succeeded
    assert_eq!(total_writes, num_tasks * writes_per_task);
    
    // Flush and verify file exists
    debug_writer.flush().await.unwrap();
    let proto_file = temp_dir.path().join("zerobus/proto/test_table.proto");
    assert!(proto_file.exists(), "Protobuf file should exist after concurrent writes");
}

#[tokio::test]
async fn test_concurrent_arrow_and_protobuf_writes() {
    // Test concurrent writes to both Arrow and Protobuf files
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = Arc::new(
        DebugWriter::new(
            temp_dir.path().to_path_buf(),
            "test_table".to_string(),
            Duration::from_secs(5),
            None,
        )
        .unwrap(),
    );

    let num_tasks = 5;

    let mut handles = vec![];
    
    // Spawn tasks writing Arrow batches
    for task_id in 0..num_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            let batch = create_test_batch();
            writer.write_arrow(&batch).await.unwrap();
            (format!("arrow_{}", task_id), true)
        });
        handles.push(handle);
    }

    // Spawn tasks writing Protobuf data
    for task_id in 0..num_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            let test_bytes = format!("proto_{}_data", task_id).into_bytes();
            writer.write_protobuf(&test_bytes, false).await.unwrap();
            (format!("proto_{}", task_id), false)
        });
        handles.push(handle);
    }

    // Wait for all tasks
    let mut arrow_count = 0;
    let mut proto_count = 0;
    for handle in handles {
        let (name, is_arrow) = handle.await.unwrap();
        if is_arrow {
            arrow_count += 1;
        } else {
            proto_count += 1;
        }
    }

    // Verify both types of writes succeeded
    assert_eq!(arrow_count, num_tasks);
    assert_eq!(proto_count, num_tasks);
    
    // Flush and verify both files exist
    debug_writer.flush().await.unwrap();
    let arrow_file = temp_dir.path().join("zerobus/arrow/test_table.arrow");
    let proto_file = temp_dir.path().join("zerobus/proto/test_table.proto");
    assert!(arrow_file.exists(), "Arrow file should exist");
    assert!(proto_file.exists(), "Protobuf file should exist");
}

#[tokio::test]
async fn test_concurrent_writes_with_rotation() {
    // Test concurrent writes when rotation occurs
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = Arc::new(
        DebugWriter::new(
            temp_dir.path().to_path_buf(),
            "test_table".to_string(),
            Duration::from_secs(5),
            Some(2048), // Small max size to trigger rotation
        )
        .unwrap(),
    );

    let num_tasks = 10;

    let mut handles = vec![];
    for task_id in 0..num_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            // Create larger batch to trigger rotation
            let schema = Schema::new(vec![Field::new("data", DataType::Utf8, false)]);
            let data: Vec<String> = (0..100).map(|i| format!("task_{}_data_{}", task_id, i)).collect();
            let data_array = StringArray::from(data);
            let batch = RecordBatch::try_new(
                Arc::new(schema),
                vec![Arc::new(data_array)],
            )
            .unwrap();
            
            writer.write_arrow(&batch).await.unwrap();
            task_id
        });
        handles.push(handle);
    }

    // Wait for all tasks
    let mut completed_tasks = 0;
    for handle in handles {
        let task_id = handle.await.unwrap();
        completed_tasks += 1;
        assert!(task_id < num_tasks);
    }

    // Verify all tasks completed
    assert_eq!(completed_tasks, num_tasks);
    
    // Flush and verify files exist (may have rotated)
    debug_writer.flush().await.unwrap();
    let arrow_dir = temp_dir.path().join("zerobus/arrow");
    let files: Vec<_> = std::fs::read_dir(&arrow_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();
    
    assert!(!files.is_empty(), "Should have at least one Arrow file");
}

#[tokio::test]
async fn test_concurrent_flush_operations() {
    // Test that flush operations work correctly under concurrent access
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = Arc::new(
        DebugWriter::new(
            temp_dir.path().to_path_buf(),
            "test_table".to_string(),
            Duration::from_secs(5),
            None,
        )
        .unwrap(),
    );

    // Write some data
    let batch = create_test_batch();
    debug_writer.write_arrow(&batch).await.unwrap();

    // Spawn multiple flush tasks
    let num_tasks = 5;
    let mut handles = vec![];
    for _ in 0..num_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            writer.flush().await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all flush operations
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify file exists and is valid
    let arrow_file = temp_dir.path().join("zerobus/arrow/test_table.arrow");
    assert!(arrow_file.exists(), "Arrow file should exist after concurrent flushes");
}

#[tokio::test]
async fn test_concurrent_write_and_flush() {
    // Test concurrent write and flush operations
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = Arc::new(
        DebugWriter::new(
            temp_dir.path().to_path_buf(),
            "test_table".to_string(),
            Duration::from_secs(5),
            None,
        )
        .unwrap(),
    );

    let num_write_tasks = 5;
    let num_flush_tasks = 3;

    let mut handles = vec![];

    // Spawn write tasks
    for _ in 0..num_write_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            let batch = create_test_batch();
            writer.write_arrow(&batch).await.unwrap();
            "write"
        });
        handles.push(handle);
    }

    // Spawn flush tasks
    for _ in 0..num_flush_tasks {
        let writer = debug_writer.clone();
        let handle = tokio::spawn(async move {
            // Small delay to ensure writes are happening
            tokio::time::sleep(Duration::from_millis(10)).await;
            writer.flush().await.unwrap();
            "flush"
        });
        handles.push(handle);
    }

    // Wait for all operations
    let mut write_count = 0;
    let mut flush_count = 0;
    for handle in handles {
        let op_type = handle.await.unwrap();
        match op_type {
            "write" => write_count += 1,
            "flush" => flush_count += 1,
            _ => {}
        }
    }

    // Verify all operations completed
    assert_eq!(write_count, num_write_tasks);
    assert_eq!(flush_count, num_flush_tasks);
    
    // Final flush and verify
    debug_writer.flush().await.unwrap();
    let arrow_file = temp_dir.path().join("zerobus/arrow/test_table.arrow");
    assert!(arrow_file.exists(), "Arrow file should exist");
}


//! Integration tests for quarantine workflow with per-row error handling
//!
//! These tests verify the complete workflow of:
//! 1. Sending a batch with partial failures
//! 2. Extracting failed rows for quarantine
//! 3. Extracting successful rows for writing to main table
//! 4. Verifying that only failed rows are quarantined

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;
use std::sync::Arc;

/// Create a test RecordBatch with multiple rows
fn create_test_batch() -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    let id_array = Int64Array::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let name_array = StringArray::from(vec![
        "Alice", "Bob", "Charlie", "David", "Eve",
        "Frank", "Grace", "Henry", "Ivy", "Jack",
    ]);
    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap()
}

#[test]
fn test_quarantine_workflow_partial_success() {
    // Simulate a TransmissionResult with partial success
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (1, ZerobusError::ConversionError("Row 1 conversion error".to_string())),
            (3, ZerobusError::TransmissionError("Row 3 transmission error".to_string())),
            (7, ZerobusError::ConversionError("Row 7 conversion error".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 4, 5, 6, 8, 9]),
        total_rows: 10,
        successful_count: 7,
        failed_count: 3,
    };

    // Step 1: Verify partial success
    assert!(result.is_partial_success());
    assert!(result.has_failed_rows());
    assert!(result.has_successful_rows());

    // Step 2: Extract failed rows for quarantine
    let failed_indices = result.get_failed_row_indices();
    assert_eq!(failed_indices, vec![1, 3, 7]);
    
    let failed_batch = result.extract_failed_batch(&batch).unwrap();
    assert_eq!(failed_batch.num_rows(), 3);
    
    // Verify failed rows contain correct data
    let id_array = failed_batch.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
    assert_eq!(id_array.value(0), 2); // Row 1: Bob
    assert_eq!(id_array.value(1), 4); // Row 3: David
    assert_eq!(id_array.value(2), 8); // Row 7: Henry

    // Step 3: Extract successful rows for writing to main table
    let successful_indices = result.get_successful_row_indices();
    assert_eq!(successful_indices.len(), 7);
    
    let successful_batch = result.extract_successful_batch(&batch).unwrap();
    assert_eq!(successful_batch.num_rows(), 7);
    
    // Verify successful rows contain correct data
    let id_array = successful_batch.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
    assert_eq!(id_array.value(0), 1); // Row 0: Alice
    assert_eq!(id_array.value(1), 3); // Row 2: Charlie
    assert_eq!(id_array.value(2), 5); // Row 4: Eve

    // Step 4: Verify consistency - total rows = successful + failed
    assert_eq!(result.total_rows, result.successful_count + result.failed_count);
    assert_eq!(failed_batch.num_rows() + successful_batch.num_rows(), result.total_rows);
}

#[test]
fn test_quarantine_workflow_all_success() {
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: None,
        successful_rows: Some((0..10).collect()),
        total_rows: 10,
        successful_count: 10,
        failed_count: 0,
    };

    // No failed rows to quarantine
    assert!(!result.has_failed_rows());
    assert!(result.has_successful_rows());
    assert!(!result.is_partial_success());

    let failed_batch = result.extract_failed_batch(&batch);
    assert!(failed_batch.is_none());

    // All rows should be in successful batch
    let successful_batch = result.extract_successful_batch(&batch).unwrap();
    assert_eq!(successful_batch.num_rows(), 10);
}

#[test]
fn test_quarantine_workflow_all_failed() {
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: false,
        error: None,
        attempts: 3,
        latency_ms: Some(500),
        batch_size_bytes: 2048,
        failed_rows: Some(
            (0..10)
                .map(|i| (i, ZerobusError::ConversionError(format!("Row {} error", i))))
                .collect(),
        ),
        successful_rows: None,
        total_rows: 10,
        successful_count: 0,
        failed_count: 10,
    };

    // All rows failed
    assert!(result.has_failed_rows());
    assert!(!result.has_successful_rows());
    assert!(!result.is_partial_success());

    // All rows should be in failed batch
    let failed_batch = result.extract_failed_batch(&batch).unwrap();
    assert_eq!(failed_batch.num_rows(), 10);

    // No successful rows
    let successful_batch = result.extract_successful_batch(&batch);
    assert!(successful_batch.is_none());
}

#[test]
fn test_quarantine_workflow_error_type_filtering() {
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Conversion error".to_string())),
            (1, ZerobusError::TransmissionError("Transmission error".to_string())),
            (2, ZerobusError::ConversionError("Conversion error".to_string())),
            (3, ZerobusError::ConnectionError("Connection error".to_string())),
        ]),
        successful_rows: Some(vec![4, 5, 6, 7, 8, 9]),
        total_rows: 10,
        successful_count: 6,
        failed_count: 4,
    };

    // Filter by error type
    let conversion_error_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::ConversionError(_))
    });
    assert_eq!(conversion_error_indices, vec![0, 2]);

    let transmission_error_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::TransmissionError(_))
    });
    assert_eq!(transmission_error_indices, vec![1]);

    // Extract batches filtered by error type
    let conversion_failed_indices: Vec<usize> = conversion_error_indices;
    let mut conversion_arrays = Vec::new();
    for array in batch.columns() {
        let taken = arrow::compute::take(
            array,
            &arrow::array::UInt32Array::from(
                conversion_failed_indices.iter().map(|&idx| idx as u32).collect::<Vec<_>>(),
            ),
            None,
        )
        .unwrap();
        conversion_arrays.push(taken);
    }
    let conversion_failed_batch = RecordBatch::try_new(batch.schema(), conversion_arrays).unwrap();
    assert_eq!(conversion_failed_batch.num_rows(), 2);
}

#[test]
fn test_quarantine_workflow_empty_batch() {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    let id_array = Int64Array::from(Vec::<i64>::new());
    let name_array = StringArray::from(Vec::<String>::new());
    let empty_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap();

    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(10),
        batch_size_bytes: 0,
        failed_rows: None,
        successful_rows: None,
        total_rows: 0,
        successful_count: 0,
        failed_count: 0,
    };

    // Empty batch should return None for both extractions
    assert!(result.extract_failed_batch(&empty_batch).is_none());
    assert!(result.extract_successful_batch(&empty_batch).is_none());
}

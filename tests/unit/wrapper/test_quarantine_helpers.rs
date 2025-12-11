//! Unit tests for quarantine workflow helper methods in TransmissionResult
//!
//! These tests verify helper methods that make it easier to quarantine
//! failed rows and extract successful rows from batches.

use arrow::array::{Int64Array, StringArray};
use arrow::compute;
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
    let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie", "David", "Eve"]);
    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap()
}

#[test]
fn test_get_failed_row_indices() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (1, ZerobusError::ConversionError("Row 1 error".to_string())),
            (3, ZerobusError::TransmissionError("Row 3 error".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    let failed_indices = result.get_failed_row_indices();
    assert_eq!(failed_indices, vec![1, 3]);
}

#[test]
fn test_get_failed_row_indices_empty() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: None,
        successful_rows: Some(vec![0, 1, 2]),
        total_rows: 3,
        successful_count: 3,
        failed_count: 0,
    };

    let failed_indices = result.get_failed_row_indices();
    assert!(failed_indices.is_empty());
}

#[test]
fn test_get_successful_row_indices() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (1, ZerobusError::ConversionError("Row 1 error".to_string())),
            (3, ZerobusError::TransmissionError("Row 3 error".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    let successful_indices = result.get_successful_row_indices();
    assert_eq!(successful_indices, vec![0, 2, 4]);
}

#[test]
fn test_get_successful_row_indices_empty() {
    let result = TransmissionResult {
        success: false,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Row 0 error".to_string())),
            (1, ZerobusError::ConversionError("Row 1 error".to_string())),
        ]),
        successful_rows: None,
        total_rows: 2,
        successful_count: 0,
        failed_count: 2,
    };

    let successful_indices = result.get_successful_row_indices();
    assert!(successful_indices.is_empty());
}

#[test]
fn test_extract_failed_batch() {
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (1, ZerobusError::ConversionError("Row 1 error".to_string())),
            (3, ZerobusError::TransmissionError("Row 3 error".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    let failed_batch = result.extract_failed_batch(&batch).unwrap();
    assert_eq!(failed_batch.num_rows(), 2);
    
    // Verify rows are in correct order (should be row 1 and row 3)
    let id_array = failed_batch.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
    assert_eq!(id_array.value(0), 2); // Row 1 from original batch (Bob)
    assert_eq!(id_array.value(1), 4); // Row 3 from original batch (David)
}

#[test]
fn test_extract_failed_batch_empty() {
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: true,
        message: "All succeeded".to_string(),
        error: None,
        failed_rows: None,
        successful_rows: Some(vec![0, 1, 2, 3, 4]),
        total_rows: 5,
        successful_count: 5,
        failed_count: 0,
        retry_attempts: 0,
        latency_ms: 100,
    };

    let failed_batch = result.extract_failed_batch(&batch);
    assert!(failed_batch.is_none());
}

#[test]
fn test_extract_successful_batch() {
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (1, ZerobusError::ConversionError("Row 1 error".to_string())),
            (3, ZerobusError::TransmissionError("Row 3 error".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    let successful_batch = result.extract_successful_batch(&batch).unwrap();
    assert_eq!(successful_batch.num_rows(), 3);
    
    // Verify rows are in correct order (should be rows 0, 2, 4)
    let id_array = successful_batch.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
    assert_eq!(id_array.value(0), 1); // Row 0 from original batch (Alice)
    assert_eq!(id_array.value(1), 3); // Row 2 from original batch (Charlie)
    assert_eq!(id_array.value(2), 5); // Row 4 from original batch (Eve)
}

#[test]
fn test_extract_successful_batch_empty() {
    let batch = create_test_batch();
    let result = TransmissionResult {
        success: false,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Row 0 error".to_string())),
            (1, ZerobusError::ConversionError("Row 1 error".to_string())),
        ]),
        successful_rows: None,
        total_rows: 2,
        successful_count: 0,
        failed_count: 2,
    };

    let successful_batch = result.extract_successful_batch(&batch);
    assert!(successful_batch.is_none());
}

#[test]
fn test_get_failed_row_indices_by_error_type() {
    let result = TransmissionResult {
        success: true,
        message: "Partial success".to_string(),
        error: None,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Row 0 conversion error".to_string())),
            (1, ZerobusError::TransmissionError("Row 1 transmission error".to_string())),
            (2, ZerobusError::ConversionError("Row 2 conversion error".to_string())),
        ]),
        successful_rows: Some(vec![3, 4]),
        total_rows: 5,
        successful_count: 2,
        failed_count: 3,
        retry_attempts: 0,
        latency_ms: 100,
    };

    let conversion_error_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::ConversionError(_))
    });
    assert_eq!(conversion_error_indices, vec![0, 2]);

    let transmission_error_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::TransmissionError(_))
    });
    assert_eq!(transmission_error_indices, vec![1]);
}

#[test]
fn test_get_failed_row_indices_by_error_type_empty() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 0,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: None,
        successful_rows: Some(vec![0, 1, 2]),
        total_rows: 3,
        successful_count: 3,
        failed_count: 0,
    };

    let indices = result.get_failed_row_indices_by_error_type(|_| true);
    assert!(indices.is_empty());
}

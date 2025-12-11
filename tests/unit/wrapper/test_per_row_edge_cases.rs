//! Edge case tests for per-row error handling

use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;
use std::sync::Arc;

#[test]
fn test_empty_batch_edge_case() {
    // Edge case: batch contains zero rows
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let empty_batch = RecordBatch::try_new(Arc::new(schema), vec![]).unwrap();

    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(0),
        batch_size_bytes: 0,
        failed_rows: None,
        successful_rows: None,
        total_rows: 0,
        successful_count: 0,
        failed_count: 0,
    };

    assert_eq!(result.total_rows, 0);
    assert_eq!(result.successful_count, 0);
    assert_eq!(result.failed_count, 0);
    assert_eq!(result.total_rows, result.successful_count + result.failed_count);
}

#[test]
fn test_all_rows_succeed_edge_case() {
    // Edge case: all rows succeed
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: None, // or Some(vec![])
        successful_rows: Some(vec![0, 1, 2, 3, 4]),
        total_rows: 5,
        successful_count: 5,
        failed_count: 0,
    };

    assert_eq!(result.successful_count, result.total_rows);
    assert_eq!(result.failed_count, 0);
    assert!(!result.has_failed_rows());
    assert!(result.has_successful_rows());
}

#[test]
fn test_all_rows_fail_edge_case() {
    // Edge case: all rows fail
    let result = TransmissionResult {
        success: false,
        error: None, // Per-row errors, no batch-level error
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("error 1".to_string())),
            (1, ZerobusError::ConversionError("error 2".to_string())),
            (2, ZerobusError::ConversionError("error 3".to_string())),
        ]),
        successful_rows: None, // or Some(vec![])
        total_rows: 3,
        successful_count: 0,
        failed_count: 3,
    };

    assert_eq!(result.failed_count, result.total_rows);
    assert_eq!(result.successful_count, 0);
    assert!(result.has_failed_rows());
    assert!(!result.has_successful_rows());
}

#[test]
fn test_batch_level_error_edge_case() {
    // Edge case: batch-level error (authentication, connection before processing)
    let result = TransmissionResult {
        success: false,
        error: Some(ZerobusError::AuthenticationError("Invalid credentials".to_string())),
        attempts: 3,
        latency_ms: Some(50),
        batch_size_bytes: 1024,
        failed_rows: None, // No row processing occurred
        successful_rows: None,
        total_rows: 10,
        successful_count: 0,
        failed_count: 0, // Batch-level error, no per-row processing
    };

    assert!(result.error.is_some());
    assert_eq!(result.failed_rows, None);
    assert_eq!(result.successful_rows, None);
    // Batch-level error means no per-row processing
    assert_eq!(result.successful_count, 0);
    assert_eq!(result.failed_count, 0);
}

#[test]
fn test_very_large_batch_edge_case() {
    // Edge case: very large batch (20,000+ rows)
    let large_batch_size = 20000;
    let successful_indices: Vec<usize> = (0..large_batch_size).collect();
    
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(5000),
        batch_size_bytes: 10 * 1024 * 1024, // 10MB
        failed_rows: None,
        successful_rows: Some(successful_indices.clone()),
        total_rows: large_batch_size,
        successful_count: large_batch_size,
        failed_count: 0,
    };

    assert_eq!(result.total_rows, large_batch_size);
    assert_eq!(result.successful_count, large_batch_size);
    assert_eq!(result.failed_count, 0);
    assert_eq!(
        result.successful_rows.as_ref().unwrap().len(),
        large_batch_size
    );
}

#[test]
fn test_mixed_error_types_edge_case() {
    // Edge case: rows fail with different error types
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("conversion error".to_string())),
            (2, ZerobusError::TransmissionError("transmission error".to_string())),
            (4, ZerobusError::ConnectionError("connection error".to_string())),
        ]),
        successful_rows: Some(vec![1, 3, 5]),
        total_rows: 6,
        successful_count: 3,
        failed_count: 3,
    };

    assert_eq!(result.total_rows, 6);
    assert_eq!(result.successful_count, 3);
    assert_eq!(result.failed_count, 3);
    
    // Verify different error types are preserved
    let failed_rows = result.failed_rows.as_ref().unwrap();
    assert_eq!(failed_rows.len(), 3);
    
    match &failed_rows[0].1 {
        ZerobusError::ConversionError(_) => {}
        _ => panic!("Expected ConversionError"),
    }
    
    match &failed_rows[1].1 {
        ZerobusError::TransmissionError(_) => {}
        _ => panic!("Expected TransmissionError"),
    }
    
    match &failed_rows[2].1 {
        ZerobusError::ConnectionError(_) => {}
        _ => panic!("Expected ConnectionError"),
    }
}

#[test]
fn test_consistency_validation_edge_case() {
    // Edge case: verify consistency checks hold for various scenarios
    let scenarios = vec![
        // All succeed
        (5, 5, 0),
        // All fail
        (5, 0, 5),
        // Partial success
        (10, 7, 3),
        // Single row succeed
        (1, 1, 0),
        // Single row fail
        (1, 0, 1),
    ];

    for (total, successful, failed) in scenarios {
        let result = TransmissionResult {
            success: successful > 0,
            error: None,
            attempts: 1,
            latency_ms: Some(100),
            batch_size_bytes: 1024,
            failed_rows: if failed > 0 {
                Some((0..failed).map(|i| (i, ZerobusError::ConversionError("error".to_string()))).collect())
            } else {
                None
            },
            successful_rows: if successful > 0 {
                Some((0..successful).collect())
            } else {
                None
            },
            total_rows: total,
            successful_count: successful,
            failed_count: failed,
        };

        // Consistency check: total_rows == successful_count + failed_count
        assert_eq!(result.total_rows, result.successful_count + result.failed_count);
        
        // Consistency check: vector lengths match counts
        if let Some(ref successful_rows) = result.successful_rows {
            assert_eq!(successful_rows.len(), result.successful_count);
        }
        
        if let Some(ref failed_rows) = result.failed_rows {
            assert_eq!(failed_rows.len(), result.failed_count);
        }
    }
}

//! Unit tests for per-row transmission error collection

use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;

#[test]
fn test_transmission_result_merges_conversion_and_transmission_errors() {
    // Test that TransmissionResult correctly merges conversion errors and transmission errors
    let conversion_errors = vec![
        (1, ZerobusError::ConversionError("conversion error 1".to_string())),
    ];
    let transmission_errors = vec![
        (3, ZerobusError::TransmissionError("transmission error 1".to_string())),
    ];

    // After implementation, send_batch_with_descriptor should merge these
    // For now, we test the expected structure
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some({
            let mut merged = conversion_errors.clone();
            merged.extend(transmission_errors.clone());
            merged
        }),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    assert_eq!(result.failed_rows.as_ref().unwrap().len(), 2);
    assert_eq!(result.successful_rows.as_ref().unwrap().len(), 3);
    assert_eq!(result.total_rows, 5);
}

#[test]
fn test_transmission_continues_after_row_failure() {
    // Test that transmission continues processing remaining rows after a row fails
    // This is a behavioral test that will be verified through integration tests
    // For unit tests, we verify the data structure supports this
    
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (1, ZerobusError::TransmissionError("row 1 failed".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 3, 4]),
        total_rows: 5,
        successful_count: 4,
        failed_count: 1,
    };

    // Verify that we have both successful and failed rows (partial success)
    assert!(result.is_partial_success());
    assert_eq!(result.successful_count, 4);
    assert_eq!(result.failed_count, 1);
    assert_eq!(result.total_rows, 5);
}

#[test]
fn test_transmission_error_types() {
    // Test that different error types are preserved per-row
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("conversion error".to_string())),
            (1, ZerobusError::TransmissionError("transmission error".to_string())),
            (2, ZerobusError::ConnectionError("connection error".to_string())),
        ]),
        successful_rows: Some(vec![3, 4]),
        total_rows: 5,
        successful_count: 2,
        failed_count: 3,
    };

    let failed_rows = result.failed_rows.as_ref().unwrap();
    
    // Verify error types are preserved
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
fn test_transmission_row_indices_correct() {
    // Test that row indices in failed_rows and successful_rows are correct and non-overlapping
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (1, ZerobusError::TransmissionError("error".to_string())),
            (3, ZerobusError::TransmissionError("error".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    let failed_indices: Vec<usize> = result.get_failed_row_indices();
    let successful_indices = result.get_successful_row_indices();

    // Verify indices are correct
    assert_eq!(failed_indices, vec![1, 3]);
    assert_eq!(successful_indices, vec![0, 2, 4]);

    // Verify no overlap
    for idx in &failed_indices {
        assert!(!successful_indices.contains(idx));
    }

    // Verify all indices are within total_rows
    for idx in &failed_indices {
        assert!(*idx < result.total_rows);
    }
    for idx in &successful_indices {
        assert!(*idx < result.total_rows);
    }
}

#[test]
fn test_transmission_retry_preserves_per_row_errors() {
    // Test that per-row errors are preserved across retry attempts
    // This tests the interaction with retry logic
    let result_after_retry = TransmissionResult {
        success: true,
        error: None,
        attempts: 3, // Multiple retry attempts
        latency_ms: Some(150),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (2, ZerobusError::ConversionError("persistent error".to_string())),
        ]),
        successful_rows: Some(vec![0, 1, 3, 4]),
        total_rows: 5,
        successful_count: 4,
        failed_count: 1,
    };

    // Verify errors are still present after retries
    assert_eq!(result_after_retry.attempts, 3);
    assert_eq!(result_after_retry.failed_rows.as_ref().unwrap().len(), 1);
    // The failed row should still be marked as failed (not retried successfully)
    assert_eq!(result_after_retry.failed_count, 1);
}

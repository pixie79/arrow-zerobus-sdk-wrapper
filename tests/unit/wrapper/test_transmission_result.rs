//! Unit tests for TransmissionResult struct extension with per-row error tracking

use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;

#[test]
fn test_transmission_result_new_fields_exist() {
    // Test that new fields exist and can be accessed
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: None,
        successful_rows: Some(vec![0, 1, 2]),
        total_rows: 3,
        successful_count: 3,
        failed_count: 0,
    };

    assert_eq!(result.total_rows, 3);
    assert_eq!(result.successful_count, 3);
    assert_eq!(result.failed_count, 0);
    assert_eq!(result.successful_rows, Some(vec![0, 1, 2]));
    assert_eq!(result.failed_rows, None);
}

#[test]
fn test_transmission_result_with_failed_rows() {
    let failed_rows = vec![
        (1, ZerobusError::ConversionError("test error 1".to_string())),
        (3, ZerobusError::TransmissionError("test error 2".to_string())),
    ];

    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(failed_rows.clone()),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    assert_eq!(result.failed_rows, Some(failed_rows));
    assert_eq!(result.successful_rows, Some(vec![0, 2, 4]));
    assert_eq!(result.total_rows, 5);
    assert_eq!(result.successful_count, 3);
    assert_eq!(result.failed_count, 2);
}

#[test]
fn test_transmission_result_consistency_all_success() {
    // Test consistency: total_rows == successful_count + failed_count
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: None,
        successful_rows: Some(vec![0, 1, 2, 3, 4]),
        total_rows: 5,
        successful_count: 5,
        failed_count: 0,
    };

    assert_eq!(result.total_rows, result.successful_count + result.failed_count);
    assert_eq!(
        result.successful_rows.as_ref().unwrap().len(),
        result.successful_count
    );
}

#[test]
fn test_transmission_result_consistency_partial_success() {
    // Test consistency with partial success
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (1, ZerobusError::ConversionError("error".to_string())),
            (3, ZerobusError::TransmissionError("error".to_string())),
        ]),
        successful_rows: Some(vec![0, 2, 4]),
        total_rows: 5,
        successful_count: 3,
        failed_count: 2,
    };

    assert_eq!(result.total_rows, result.successful_count + result.failed_count);
    assert_eq!(
        result.successful_rows.as_ref().unwrap().len(),
        result.successful_count
    );
    assert_eq!(
        result.failed_rows.as_ref().unwrap().len(),
        result.failed_count
    );
}

#[test]
fn test_transmission_result_consistency_all_failed() {
    // Test consistency when all rows fail
    let result = TransmissionResult {
        success: false,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("error 1".to_string())),
            (1, ZerobusError::ConversionError("error 2".to_string())),
            (2, ZerobusError::ConversionError("error 3".to_string())),
        ]),
        successful_rows: None,
        total_rows: 3,
        successful_count: 0,
        failed_count: 3,
    };

    assert_eq!(result.total_rows, result.successful_count + result.failed_count);
    assert_eq!(
        result.failed_rows.as_ref().unwrap().len(),
        result.failed_count
    );
}

#[test]
fn test_transmission_result_empty_batch() {
    // Test edge case: empty batch
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
fn test_transmission_result_batch_level_error() {
    // Test batch-level error (authentication failure before row processing)
    let result = TransmissionResult {
        success: false,
        error: Some(ZerobusError::AuthenticationError(
            "Invalid credentials".to_string(),
        )),
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
}

#[test]
fn test_transmission_result_backward_compatibility() {
    // Test that existing code patterns still work
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: None,
        successful_rows: None,
        total_rows: 5,
        successful_count: 5,
        failed_count: 0,
    };

    // Existing code that checks success should still work
    if result.success {
        assert!(result.error.is_none());
        assert!(result.latency_ms.is_some());
    }
}

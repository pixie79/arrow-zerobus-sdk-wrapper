//! Contract tests for TransmissionResult structure

use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;

#[test]
fn test_transmission_result_contract_all_fields() {
    // Contract: TransmissionResult must have all required fields
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: None,
        successful_rows: Some(vec![0, 1]),
        total_rows: 2,
        successful_count: 2,
        failed_count: 0,
    };

    // Verify all fields are accessible
    let _ = result.success;
    let _ = result.error;
    let _ = result.attempts;
    let _ = result.latency_ms;
    let _ = result.batch_size_bytes;
    let _ = result.failed_rows;
    let _ = result.successful_rows;
    let _ = result.total_rows;
    let _ = result.successful_count;
    let _ = result.failed_count;
}

#[test]
fn test_transmission_result_contract_consistency() {
    // Contract: total_rows must equal successful_count + failed_count
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![(1, ZerobusError::ConversionError("error".to_string()))]),
        successful_rows: Some(vec![0, 2]),
        total_rows: 3,
        successful_count: 2,
        failed_count: 1,
    };

    assert_eq!(result.total_rows, result.successful_count + result.failed_count);
}

#[test]
fn test_transmission_result_contract_vector_lengths() {
    // Contract: Vector lengths must match counts
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 1024,
        failed_rows: Some(vec![(1, ZerobusError::ConversionError("error".to_string()))]),
        successful_rows: Some(vec![0, 2]),
        total_rows: 3,
        successful_count: 2,
        failed_count: 1,
    };

    if let Some(ref successful) = result.successful_rows {
        assert_eq!(successful.len(), result.successful_count);
    }

    if let Some(ref failed) = result.failed_rows {
        assert_eq!(failed.len(), result.failed_count);
    }
}

#[test]
fn test_transmission_result_contract_backward_compatibility() {
    // Contract: Existing code using success and error fields must continue to work
    let result = TransmissionResult {
        success: false,
        error: Some(ZerobusError::AuthenticationError("auth failed".to_string())),
        attempts: 3,
        latency_ms: Some(50),
        batch_size_bytes: 512,
        failed_rows: None,
        successful_rows: None,
        total_rows: 0,
        successful_count: 0,
        failed_count: 0,
    };

    // Existing pattern: check success and error
    if !result.success {
        assert!(result.error.is_some());
    }
}

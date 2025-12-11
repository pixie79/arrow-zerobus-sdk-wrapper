//! Unit tests for error analysis and pattern analysis helpers in TransmissionResult
//!
//! These tests verify helper methods that enable error pattern analysis,
//! error statistics, and debugging capabilities.

use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::{ErrorStatistics, TransmissionResult};
use std::collections::HashMap;

#[test]
fn test_group_errors_by_type() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Conversion error 1".to_string())),
            (1, ZerobusError::TransmissionError("Transmission error 1".to_string())),
            (2, ZerobusError::ConversionError("Conversion error 2".to_string())),
            (3, ZerobusError::ConnectionError("Connection error 1".to_string())),
            (4, ZerobusError::ConversionError("Conversion error 3".to_string())),
        ]),
        successful_rows: Some(vec![5, 6, 7, 8, 9]),
        total_rows: 10,
        successful_count: 5,
        failed_count: 5,
    };

    let grouped = result.group_errors_by_type();
    
    assert_eq!(grouped.len(), 3);
    assert_eq!(grouped.get("ConversionError"), Some(&vec![0, 2, 4]));
    assert_eq!(grouped.get("TransmissionError"), Some(&vec![1]));
    assert_eq!(grouped.get("ConnectionError"), Some(&vec![3]));
}

#[test]
fn test_group_errors_by_type_empty() {
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

    let grouped = result.group_errors_by_type();
    assert!(grouped.is_empty());
}

#[test]
fn test_get_error_statistics() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Error 1".to_string())),
            (1, ZerobusError::TransmissionError("Error 2".to_string())),
            (2, ZerobusError::ConversionError("Error 3".to_string())),
            (3, ZerobusError::ConnectionError("Error 4".to_string())),
            (4, ZerobusError::ConversionError("Error 5".to_string())),
        ]),
        successful_rows: Some(vec![5, 6, 7, 8, 9]),
        total_rows: 10,
        successful_count: 5,
        failed_count: 5,
    };

    let stats = result.get_error_statistics();
    
    assert_eq!(stats.total_rows, 10);
    assert_eq!(stats.successful_count, 5);
    assert_eq!(stats.failed_count, 5);
    assert_eq!(stats.success_rate, 0.5);
    assert_eq!(stats.failure_rate, 0.5);
    
    assert_eq!(stats.error_type_counts.get("ConversionError"), Some(&3));
    assert_eq!(stats.error_type_counts.get("TransmissionError"), Some(&1));
    assert_eq!(stats.error_type_counts.get("ConnectionError"), Some(&1));
}

#[test]
fn test_get_error_statistics_all_success() {
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

    let stats = result.get_error_statistics();
    
    assert_eq!(stats.total_rows, 5);
    assert_eq!(stats.successful_count, 5);
    assert_eq!(stats.failed_count, 0);
    assert_eq!(stats.success_rate, 1.0);
    assert_eq!(stats.failure_rate, 0.0);
    assert!(stats.error_type_counts.is_empty());
}

#[test]
fn test_get_error_statistics_all_failed() {
    let result = TransmissionResult {
        success: false,
        error: None,
        attempts: 3,
        latency_ms: Some(500),
        batch_size_bytes: 1024,
        failed_rows: Some(
            (0..5)
                .map(|i| (i, ZerobusError::ConversionError(format!("Error {}", i))))
                .collect(),
        ),
        successful_rows: None,
        total_rows: 5,
        successful_count: 0,
        failed_count: 5,
    };

    let stats = result.get_error_statistics();
    
    assert_eq!(stats.total_rows, 5);
    assert_eq!(stats.successful_count, 0);
    assert_eq!(stats.failed_count, 5);
    assert_eq!(stats.success_rate, 0.0);
    assert_eq!(stats.failure_rate, 1.0);
    assert_eq!(stats.error_type_counts.get("ConversionError"), Some(&5));
}

#[test]
fn test_get_failed_row_indices_by_error_type_already_exists() {
    // This method already exists from User Story 2, but we test it here for completeness
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Error 1".to_string())),
            (1, ZerobusError::TransmissionError("Error 2".to_string())),
            (2, ZerobusError::ConversionError("Error 3".to_string())),
        ]),
        successful_rows: Some(vec![3, 4]),
        total_rows: 5,
        successful_count: 2,
        failed_count: 3,
    };

    let conversion_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::ConversionError(_))
    });
    assert_eq!(conversion_indices, vec![0, 2]);

    let transmission_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::TransmissionError(_))
    });
    assert_eq!(transmission_indices, vec![1]);
}

#[test]
fn test_get_error_messages() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Field 'name' type mismatch".to_string())),
            (1, ZerobusError::TransmissionError("Network timeout".to_string())),
            (2, ZerobusError::ConversionError("Field 'age' missing required value".to_string())),
        ]),
        successful_rows: Some(vec![3, 4]),
        total_rows: 5,
        successful_count: 2,
        failed_count: 3,
    };

    let error_messages = result.get_error_messages();
    
    assert_eq!(error_messages.len(), 3);
    assert!(error_messages.contains(&"Field 'name' type mismatch".to_string()));
    assert!(error_messages.contains(&"Network timeout".to_string()));
    assert!(error_messages.contains(&"Field 'age' missing required value".to_string()));
}

#[test]
fn test_get_error_messages_empty() {
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

    let error_messages = result.get_error_messages();
    assert!(error_messages.is_empty());
}

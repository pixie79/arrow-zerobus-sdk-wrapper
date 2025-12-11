//! Integration tests for error analysis and pattern analysis
//!
//! These tests verify error analysis capabilities in realistic scenarios
//! with multiple batches and error patterns.

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::{ErrorStatistics, TransmissionResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Create a test RecordBatch
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
fn test_error_pattern_analysis_multiple_batches() {
    // Simulate multiple batches with different error patterns
    let batch1_result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Field 'age' type mismatch".to_string())),
            (1, ZerobusError::ConversionError("Field 'age' type mismatch".to_string())),
            (2, ZerobusError::TransmissionError("Network timeout".to_string())),
        ]),
        successful_rows: Some(vec![3, 4, 5, 6, 7]),
        total_rows: 8,
        successful_count: 5,
        failed_count: 3,
    };

    let batch2_result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(120),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Field 'age' type mismatch".to_string())),
            (1, ZerobusError::ConnectionError("Connection lost".to_string())),
        ]),
        successful_rows: Some(vec![2, 3, 4, 5, 6]),
        total_rows: 7,
        successful_count: 5,
        failed_count: 2,
    };

    // Analyze error patterns across batches
    let mut all_conversion_errors = Vec::new();
    let mut all_transmission_errors = Vec::new();
    let mut all_connection_errors = Vec::new();

    for result in &[batch1_result, batch2_result] {
        let grouped = result.group_errors_by_type();
        
        if let Some(indices) = grouped.get("ConversionError") {
            all_conversion_errors.extend(indices);
        }
        if let Some(indices) = grouped.get("TransmissionError") {
            all_transmission_errors.extend(indices);
        }
        if let Some(indices) = grouped.get("ConnectionError") {
            all_connection_errors.extend(indices);
        }
    }

    // Verify pattern detection
    assert_eq!(all_conversion_errors.len(), 3); // 2 from batch1, 1 from batch2
    assert_eq!(all_transmission_errors.len(), 1);
    assert_eq!(all_connection_errors.len(), 1);

    // Common pattern: "Field 'age' type mismatch" appears in both batches
    let batch1_messages = batch1_result.get_error_messages();
    let batch2_messages = batch2_result.get_error_messages();
    
    let common_error = "Field 'age' type mismatch";
    assert!(batch1_messages.iter().any(|m| m.contains(common_error)));
    assert!(batch2_messages.iter().any(|m| m.contains(common_error)));
}

#[test]
fn test_error_statistics_aggregation() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Error 1".to_string())),
            (1, ZerobusError::ConversionError("Error 2".to_string())),
            (2, ZerobusError::TransmissionError("Error 3".to_string())),
            (3, ZerobusError::ConnectionError("Error 4".to_string())),
            (4, ZerobusError::ConversionError("Error 5".to_string())),
        ]),
        successful_rows: Some(vec![5, 6, 7, 8, 9]),
        total_rows: 10,
        successful_count: 5,
        failed_count: 5,
    };

    let stats = result.get_error_statistics();

    // Verify statistics
    assert_eq!(stats.total_rows, 10);
    assert_eq!(stats.successful_count, 5);
    assert_eq!(stats.failed_count, 5);
    assert_eq!(stats.success_rate, 0.5);
    assert_eq!(stats.failure_rate, 0.5);

    // Verify error type distribution
    assert_eq!(stats.error_type_counts.get("ConversionError"), Some(&3));
    assert_eq!(stats.error_type_counts.get("TransmissionError"), Some(&1));
    assert_eq!(stats.error_type_counts.get("ConnectionError"), Some(&1));
    assert_eq!(stats.error_type_counts.len(), 3);
}

#[test]
fn test_error_analysis_for_monitoring() {
    // Simulate monitoring scenario: track failure rates over time
    let results = vec![
        TransmissionResult {
            success: true,
            error: None,
            attempts: 1,
            latency_ms: Some(100),
            batch_size_bytes: 1024,
            failed_rows: Some(vec![(0, ZerobusError::ConversionError("Error".to_string()))]),
            successful_rows: Some(vec![1, 2, 3, 4]),
            total_rows: 5,
            successful_count: 4,
            failed_count: 1,
        },
        TransmissionResult {
            success: true,
            error: None,
            attempts: 1,
            latency_ms: Some(110),
            batch_size_bytes: 1024,
            failed_rows: Some(vec![(0, ZerobusError::ConversionError("Error".to_string()))]),
            successful_rows: Some(vec![1, 2, 3]),
            total_rows: 4,
            successful_count: 3,
            failed_count: 1,
        },
        TransmissionResult {
            success: true,
            error: None,
            attempts: 1,
            latency_ms: Some(105),
            batch_size_bytes: 1024,
            failed_rows: None,
            successful_rows: Some(vec![0, 1, 2, 3, 4]),
            total_rows: 5,
            successful_count: 5,
            failed_count: 0,
        },
    ];

    // Calculate aggregate statistics
    let mut total_rows = 0;
    let mut total_successful = 0;
    let mut total_failed = 0;
    let mut error_type_totals: HashMap<String, usize> = HashMap::new();

    for result in &results {
        let stats = result.get_error_statistics();
        total_rows += stats.total_rows;
        total_successful += stats.successful_count;
        total_failed += stats.failed_count;

        for (error_type, count) in &stats.error_type_counts {
            *error_type_totals.entry(error_type.clone()).or_insert(0) += count;
        }
    }

    // Verify aggregate statistics
    assert_eq!(total_rows, 14);
    assert_eq!(total_successful, 12);
    assert_eq!(total_failed, 2);
    
    let overall_success_rate = total_successful as f64 / total_rows as f64;
    assert!((overall_success_rate - 0.857).abs() < 0.01); // ~85.7%

    // Verify error type totals
    assert_eq!(error_type_totals.get("ConversionError"), Some(&2));
}

#[test]
fn test_error_message_analysis() {
    let result = TransmissionResult {
        success: true,
        error: None,
        attempts: 1,
        latency_ms: Some(100),
        batch_size_bytes: 2048,
        failed_rows: Some(vec![
            (0, ZerobusError::ConversionError("Field 'name' type mismatch: expected String, got Int64".to_string())),
            (1, ZerobusError::ConversionError("Field 'age' missing required value".to_string())),
            (2, ZerobusError::TransmissionError("Network timeout after 30s".to_string())),
        ]),
        successful_rows: Some(vec![3, 4]),
        total_rows: 5,
        successful_count: 2,
        failed_count: 3,
    };

    let error_messages = result.get_error_messages();

    // Verify all error messages are captured
    assert_eq!(error_messages.len(), 3);
    assert!(error_messages.iter().any(|m| m.contains("Field 'name'")));
    assert!(error_messages.iter().any(|m| m.contains("Field 'age'")));
    assert!(error_messages.iter().any(|m| m.contains("Network timeout")));

    // Verify error messages contain sufficient detail for debugging
    let name_error = error_messages.iter().find(|m| m.contains("Field 'name'")).unwrap();
    assert!(name_error.contains("type mismatch"));
    assert!(name_error.contains("expected String"));
    assert!(name_error.contains("got Int64"));
}

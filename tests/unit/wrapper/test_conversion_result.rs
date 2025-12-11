//! Unit tests for ProtobufConversionResult struct

use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::conversion::ProtobufConversionResult;

#[test]
fn test_protobuf_conversion_result_structure() {
    // Test that ProtobufConversionResult uses ZerobusError for failed_rows
    let failed_rows = vec![
        (1, ZerobusError::ConversionError("field encoding failed".to_string())),
        (3, ZerobusError::ConversionError("type mismatch".to_string())),
    ];

    let successful_bytes = vec![
        (0, vec![1, 2, 3]),
        (2, vec![4, 5, 6]),
        (4, vec![7, 8, 9]),
    ];

    let result = ProtobufConversionResult {
        successful_bytes: successful_bytes.clone(),
        failed_rows: failed_rows.clone(),
    };

    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 2);
    assert_eq!(result.successful_bytes, successful_bytes);
    assert_eq!(result.failed_rows, failed_rows);
}

#[test]
fn test_protobuf_conversion_result_all_success() {
    let result = ProtobufConversionResult {
        successful_bytes: vec![
            (0, vec![1, 2]),
            (1, vec![3, 4]),
            (2, vec![5, 6]),
        ],
        failed_rows: vec![],
    };

    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 0);
}

#[test]
fn test_protobuf_conversion_result_all_failed() {
    let result = ProtobufConversionResult {
        successful_bytes: vec![],
        failed_rows: vec![
            (0, ZerobusError::ConversionError("error 1".to_string())),
            (1, ZerobusError::ConversionError("error 2".to_string())),
            (2, ZerobusError::ConversionError("error 3".to_string())),
        ],
    };

    assert_eq!(result.successful_bytes.len(), 0);
    assert_eq!(result.failed_rows.len(), 3);
}

#[test]
fn test_protobuf_conversion_result_partial_success() {
    let result = ProtobufConversionResult {
        successful_bytes: vec![(0, vec![1, 2]), (2, vec![5, 6])],
        failed_rows: vec![(1, ZerobusError::ConversionError("error".to_string()))],
    };

    assert_eq!(result.successful_bytes.len(), 2);
    assert_eq!(result.failed_rows.len(), 1);
}

#[test]
fn test_protobuf_conversion_result_empty() {
    let result = ProtobufConversionResult {
        successful_bytes: vec![],
        failed_rows: vec![],
    };

    assert_eq!(result.successful_bytes.len(), 0);
    assert_eq!(result.failed_rows.len(), 0);
}

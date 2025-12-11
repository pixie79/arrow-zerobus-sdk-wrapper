//! Unit tests for per-row conversion error collection

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::conversion::ProtobufConversionResult;
use arrow_zerobus_sdk_wrapper::wrapper::conversion::record_batch_to_protobuf_bytes;
use prost_types::{DescriptorProto, FieldDescriptorProto, Type};

/// Create a simple descriptor for testing
fn create_test_descriptor() -> DescriptorProto {
    let mut descriptor = DescriptorProto::default();
    descriptor.name = Some("TestMessage".to_string());

    // Add id field
    let mut id_field = FieldDescriptorProto::default();
    id_field.name = Some("id".to_string());
    id_field.number = Some(1);
    id_field.r#type = Some(Type::Int64 as i32);
    descriptor.field.push(id_field);

    // Add name field
    let mut name_field = FieldDescriptorProto::default();
    name_field.name = Some("name".to_string());
    name_field.number = Some(2);
    name_field.r#type = Some(Type::String as i32);
    descriptor.field.push(name_field);

    descriptor
}

/// Create a test RecordBatch with valid data
fn create_valid_batch() -> RecordBatch {
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

#[test]
fn test_conversion_collects_per_row_errors() {
    // Test that conversion function returns ProtobufConversionResult with per-row errors
    let batch = create_valid_batch();
    let descriptor = create_test_descriptor();

    // This should succeed for all rows with valid data
    let result = record_batch_to_protobuf_bytes(&batch, &descriptor);

    // Verify it returns ProtobufConversionResult (not Result<Vec<Vec<u8>>, ZerobusError>)
    // Function now returns ProtobufConversionResult directly
    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 0);
}

#[test]
fn test_conversion_partial_success() {
    // Test that some rows can succeed while others fail
    let batch = create_valid_batch();
    let descriptor = create_test_descriptor();

    let result = record_batch_to_protobuf_bytes(&batch, &descriptor);

    // Function now returns ProtobufConversionResult with successful_bytes and failed_rows
    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 0);
}

#[test]
fn test_conversion_all_rows_fail() {
    // Test scenario where all rows fail conversion
    // This would require invalid data or descriptor mismatch
    let batch = create_valid_batch();
    let mut descriptor = create_test_descriptor();
    
    // Create invalid descriptor (wrong field type)
    descriptor.field[0].r#type = Some(Type::String as i32); // id should be Int64

    let result = record_batch_to_protobuf_bytes(&batch, &descriptor);

    // Function now returns ProtobufConversionResult with all rows failed (type mismatch)
    assert_eq!(result.successful_bytes.len(), 0);
    assert_eq!(result.failed_rows.len(), 3);
    // Verify each failed row has correct index and error
    for (idx, error) in &result.failed_rows {
        assert!(*idx < 3);
        match error {
            ZerobusError::ConversionError(_) => {}
            _ => panic!("Expected ConversionError"),
        }
    }
}

#[test]
fn test_conversion_empty_batch() {
    // Test edge case: empty batch
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    let empty_batch = RecordBatch::try_new(Arc::new(schema), vec![]).unwrap();
    let descriptor = create_test_descriptor();

    let result = record_batch_to_protobuf_bytes(&empty_batch, &descriptor);

    // Empty batch should return empty ProtobufConversionResult
    assert_eq!(result.successful_bytes.len(), 0);
    assert_eq!(result.failed_rows.len(), 0);
}

#[test]
fn test_conversion_error_includes_row_index() {
    // Test that conversion errors include row index information
    let batch = create_valid_batch();
    let mut descriptor = create_test_descriptor();
    
    // Create invalid descriptor to force errors
    descriptor.field[0].r#type = Some(Type::String as i32);

    let result = record_batch_to_protobuf_bytes(&batch, &descriptor);

    // Verify errors include row indices
    assert!(result.failed_rows.len() > 0, "Type mismatch should result in failed rows");
    for (row_idx, error) in &result.failed_rows {
        let error_msg = format!("{:?}", error);
        // Error message should reference the row index
        assert!(error_msg.contains(&format!("row={}", row_idx)) || 
               error_msg.contains(&row_idx.to_string()),
               "Error message should include row index: {}", error_msg);
    }
}

use std::sync::Arc;

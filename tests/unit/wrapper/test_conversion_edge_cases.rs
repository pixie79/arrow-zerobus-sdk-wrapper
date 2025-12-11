//! Unit tests for conversion edge cases
//!
//! Tests for empty batches, large batches, all null values, mismatched schemas, etc.

use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::wrapper::conversion;
use arrow_zerobus_sdk_wrapper::ZerobusError;
use prost_types::{
    field_descriptor_proto::{Label, Type},
    DescriptorProto, FieldDescriptorProto,
};
use std::sync::Arc;

#[test]
fn test_empty_batch() {
    // Test RecordBatch with 0 rows
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let id_array = Int64Array::from(Vec::<i64>::new());
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array)],
    ).unwrap();

    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("id".to_string()),
            number: Some(1),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::Int64 as i32),
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    assert_eq!(result.successful_bytes.len(), 0, "Empty batch should produce empty result");
    assert_eq!(result.failed_rows.len(), 0);
}

#[test]
fn test_large_batch() {
    // Test RecordBatch with many rows (1K rows for unit test)
    // In production, this would test 1M+ rows
    let num_rows = 1000;
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    
    let ids: Vec<i64> = (0..num_rows).map(|i| i as i64).collect();
    let id_array = Int64Array::from(ids);
    
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array)],
    ).unwrap();

    assert_eq!(batch.num_rows(), num_rows);

    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("id".to_string()),
            number: Some(1),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::Int64 as i32),
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    assert_eq!(result.successful_bytes.len(), num_rows);
    assert_eq!(result.failed_rows.len(), 0);
    // Sort by row index and extract bytes
    let mut bytes_list: Vec<(usize, Vec<u8>)> = result.successful_bytes;
    bytes_list.sort_by_key(|(idx, _)| *idx);
    let bytes_list: Vec<Vec<u8>> = bytes_list.into_iter().map(|(_, bytes)| bytes).collect();
    
    // Verify all rows have bytes
    for (idx, bytes) in bytes_list.iter().enumerate() {
        assert!(!bytes.is_empty(), "Row {} should have bytes", idx);
    }
}

#[test]
fn test_all_null_values() {
    // Test batch where all values are null
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]);

    let id_array = Int64Array::from(vec![None, None, None]);
    let name_array = StringArray::from(vec![None, None, None]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    ).unwrap();

    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("id".to_string()),
                number: Some(1),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::Int64 as i32),
                type_name: None,
                extendee: None,
                default_value: None,
                oneof_index: None,
                json_name: None,
                options: None,
                proto3_optional: None,
            },
            FieldDescriptorProto {
                name: Some("name".to_string()),
                number: Some(2),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::String as i32),
                type_name: None,
                extendee: None,
                default_value: None,
                oneof_index: None,
                json_name: None,
                options: None,
                proto3_optional: None,
            },
        ],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 0);
    // Sort by row index and extract bytes
    let mut bytes_list: Vec<(usize, Vec<u8>)> = result.successful_bytes;
    bytes_list.sort_by_key(|(idx, _)| *idx);
    let bytes_list: Vec<Vec<u8>> = bytes_list.into_iter().map(|(_, bytes)| bytes).collect();
    
    // All null values should produce minimal or empty bytes (null fields are skipped)
    for (idx, bytes) in bytes_list.iter().enumerate() {
        // Null fields are skipped in Protobuf, so bytes might be empty or minimal
        // This is expected behavior
        assert!(bytes.len() <= 10, "Row {} with all nulls should have minimal bytes", idx);
    }
}

#[test]
fn test_mismatched_schema_descriptor() {
    // Test when Arrow schema doesn't match descriptor
    // Arrow has field "id" but descriptor has "name"
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let id_array = Int64Array::from(vec![1, 2, 3]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array)],
    ).unwrap();

    // Descriptor has "name" field, but Arrow has "id"
    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("name".to_string()), // Mismatch!
            number: Some(1),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::String as i32),
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    
    // Should fail or handle gracefully
    // The conversion might skip the field (since it's not in descriptor)
    // or it might fail - both are acceptable behaviors
    match result {
        Ok(bytes_list) => {
            // If it succeeds, the "id" field is skipped (not in descriptor)
            assert_eq!(bytes_list.len(), 3);
            // Bytes might be empty since no matching fields
        }
        Err(e) => {
            // Expected if mismatch causes error
            assert!(
                matches!(e, ZerobusError::ConversionError(_)),
                "Expected ConversionError, got: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_missing_fields_in_descriptor() {
    // Test when Arrow has fields not in descriptor
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("extra", DataType::Float64, false), // Not in descriptor
    ]);

    let id_array = Int64Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);
    let extra_array = Float64Array::from(vec![1.0, 2.0, 3.0]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array),
            Arc::new(name_array),
            Arc::new(extra_array),
        ],
    ).unwrap();

    // Descriptor only has "id" and "name", missing "extra"
    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("id".to_string()),
                number: Some(1),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::Int64 as i32),
                type_name: None,
                extendee: None,
                default_value: None,
                oneof_index: None,
                json_name: None,
                options: None,
                proto3_optional: None,
            },
            FieldDescriptorProto {
                name: Some("name".to_string()),
                number: Some(2),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::String as i32),
                type_name: None,
                extendee: None,
                default_value: None,
                oneof_index: None,
                json_name: None,
                options: None,
                proto3_optional: None,
            },
            // Missing "extra" field
        ],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    
    // Should succeed - extra fields are simply skipped
    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 0);
    // Sort by row index and extract bytes
    let mut bytes_list: Vec<(usize, Vec<u8>)> = result.successful_bytes;
    bytes_list.sort_by_key(|(idx, _)| *idx);
    let bytes_list: Vec<Vec<u8>> = bytes_list.into_iter().map(|(_, bytes)| bytes).collect();
    
    // Bytes should contain id and name, but not extra
    for bytes in bytes_list {
        assert!(!bytes.is_empty(), "Should have bytes for id and name fields");
    }
}

#[test]
fn test_extra_fields_in_descriptor() {
    // Test when descriptor has fields not in Arrow schema
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let id_array = Int64Array::from(vec![1, 2, 3]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array)],
    ).unwrap();

    // Descriptor has "id" and "name", but Arrow only has "id"
    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("id".to_string()),
                number: Some(1),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::Int64 as i32),
                type_name: None,
                extendee: None,
                default_value: None,
                oneof_index: None,
                json_name: None,
                options: None,
                proto3_optional: None,
            },
            FieldDescriptorProto {
                name: Some("name".to_string()), // Not in Arrow schema
                number: Some(2),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::String as i32),
                type_name: None,
                extendee: None,
                default_value: None,
                oneof_index: None,
                json_name: None,
                options: None,
                proto3_optional: None,
            },
        ],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    
    // Should succeed - missing fields are treated as null/optional
    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 0);
    // Sort by row index and extract bytes
    let mut bytes_list: Vec<(usize, Vec<u8>)> = result.successful_bytes;
    bytes_list.sort_by_key(|(idx, _)| *idx);
    let bytes_list: Vec<Vec<u8>> = bytes_list.into_iter().map(|(_, bytes)| bytes).collect();
    
    // Bytes should contain id field, name field is skipped (not in Arrow)
    for bytes in bytes_list {
        assert!(!bytes.is_empty(), "Should have bytes for id field");
    }
}

#[test]
fn test_type_mismatch() {
    // Test when Arrow type doesn't match descriptor type
    // Arrow has Int64 but descriptor expects String
    let schema = Schema::new(vec![Field::new("value", DataType::Int64, false)]);
    let value_array = Int64Array::from(vec![1, 2, 3]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(value_array)],
    ).unwrap();

    // Descriptor expects String, but Arrow has Int64
    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("value".to_string()),
            number: Some(1),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::String as i32), // Mismatch: expects String
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    
    // Should have failed rows (type mismatch)
    assert!(result.failed_rows.len() > 0, "Type mismatch should result in failed rows");
    // Check conversion errors
    for (_, error) in &result.failed_rows {
        match error {
            ZerobusError::ConversionError(msg) => {
                // Error should mention type mismatch or conversion issue
                assert!(
                    msg.contains("type") || msg.contains("conversion") || msg.contains("Int64") || msg.contains("String") || msg.contains("encoding"),
                    "Error message should mention type/conversion: {}",
                    msg
                );
            }
            _ => panic!("Expected ConversionError, got {:?}", error),
        }
    }
}

#[test]
fn test_single_row_batch() {
    // Test batch with exactly one row
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let id_array = Int64Array::from(vec![42]);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array)],
    ).unwrap();

    assert_eq!(batch.num_rows(), 1);

    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("id".to_string()),
            number: Some(1),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::Int64 as i32),
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    assert_eq!(result.successful_bytes.len(), 1);
    assert_eq!(result.failed_rows.len(), 0);
    let bytes_list: Vec<Vec<u8>> = result.successful_bytes.into_iter().map(|(_, bytes)| bytes).collect();
    assert!(!bytes_list[0].is_empty());
}

#[test]
fn test_many_columns() {
    // Test batch with many columns (20 columns)
    let num_cols = 20;
    let mut fields = vec![];
    let mut arrays = vec![];
    
    for i in 0..num_cols {
        fields.push(Field::new(format!("col_{}", i), DataType::Int64, false));
        arrays.push(Arc::new(Int64Array::from(vec![i as i64, (i + 1) as i64, (i + 2) as i64])) as Arc<dyn arrow::array::Array>);
    }
    
    let schema = Schema::new(fields);
    let batch = RecordBatch::try_new(Arc::new(schema), arrays).unwrap();
    
    assert_eq!(batch.num_rows(), 3);
    assert_eq!(batch.num_columns(), num_cols);
    
    // Create descriptor with all fields
    let mut descriptor_fields = vec![];
    for i in 0..num_cols {
        descriptor_fields.push(FieldDescriptorProto {
            name: Some(format!("col_{}", i)),
            number: Some((i + 1) as i32),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::Int64 as i32),
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        });
    }
    
    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: descriptor_fields,
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };
    
    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);
    assert_eq!(result.successful_bytes.len(), 3);
    assert_eq!(result.failed_rows.len(), 0);
    // Sort by row index and extract bytes
    let mut bytes_list: Vec<(usize, Vec<u8>)> = result.successful_bytes;
    bytes_list.sort_by_key(|(idx, _)| *idx);
    let bytes_list: Vec<Vec<u8>> = bytes_list.into_iter().map(|(_, bytes)| bytes).collect();
    
    // Each row should have bytes for all columns
    for (idx, bytes) in bytes_list.iter().enumerate() {
        assert!(!bytes.is_empty(), "Row {} should have bytes for all {} columns", idx, num_cols);
    }
}


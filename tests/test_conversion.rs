//! Integration tests for Arrow to Protobuf conversion

use arrow::array::{BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::wrapper::conversion;
use prost_types::{
    field_descriptor_proto::{Label, Type},
    DescriptorProto, FieldDescriptorProto,
};
use std::sync::Arc;

fn create_test_batch() -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("score", DataType::Float64, true),
    ]);

    let id_array = Int64Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);
    let score_array = Float64Array::from(vec![Some(95.5), None, Some(87.0)]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array),
            Arc::new(name_array),
            Arc::new(score_array),
        ],
    )
    .unwrap()
}

fn create_test_descriptor() -> DescriptorProto {
    DescriptorProto {
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
            FieldDescriptorProto {
                name: Some("score".to_string()),
                number: Some(3),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::Double as i32),
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
    }
}

#[test]
fn test_generate_protobuf_descriptor() {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let descriptor = conversion::generate_protobuf_descriptor(&schema).unwrap();

    assert_eq!(descriptor.name, Some("ZerobusMessage".to_string()));
    assert_eq!(descriptor.field.len(), 2);
    assert_eq!(descriptor.field[0].name, Some("id".to_string()));
    assert_eq!(descriptor.field[0].number, Some(1));
    assert_eq!(descriptor.field[1].name, Some("name".to_string()));
    assert_eq!(descriptor.field[1].number, Some(2));
}

#[test]
fn test_record_batch_to_protobuf_bytes() {
    let batch = create_test_batch();
    let descriptor = create_test_descriptor();

    let result = conversion::record_batch_to_protobuf_bytes(&batch, &descriptor);

    assert!(result.is_ok());
    let bytes_list = result.unwrap();
    assert_eq!(bytes_list.len(), 3); // One per row

    // Each row should have some bytes (not empty)
    for (idx, bytes) in bytes_list.iter().enumerate() {
        assert!(
            !bytes.is_empty(),
            "Row {} should have non-empty Protobuf bytes",
            idx
        );
    }
}

#[test]
fn test_record_batch_to_protobuf_bytes_empty_batch() {
    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let id_array = Int64Array::from(Vec::<i64>::new());
    let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(id_array)]).unwrap();

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
    assert!(result.is_ok());
    let bytes_list = result.unwrap();
    assert_eq!(bytes_list.len(), 0);
}

#[test]
fn test_record_batch_to_protobuf_bytes_with_nulls() {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]);

    let id_array = Int64Array::from(vec![Some(1), None, Some(3)]);
    let name_array = StringArray::from(vec![Some("Alice"), Some("Bob"), None]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap();

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
    assert!(result.is_ok());
    let bytes_list = result.unwrap();
    assert_eq!(bytes_list.len(), 3);

    // Null fields should be skipped in Protobuf encoding
    // Row 0: id=1, name="Alice" -> should have bytes
    assert!(!bytes_list[0].is_empty());
    // Row 1: id=null, name="Bob" -> should have bytes (name field)
    assert!(!bytes_list[1].is_empty());
    // Row 2: id=3, name=null -> should have bytes (id field)
    assert!(!bytes_list[2].is_empty());
}

#[test]
fn test_generate_descriptor_boolean() {
    let schema = Schema::new(vec![Field::new("active", DataType::Boolean, false)]);

    let descriptor = conversion::generate_protobuf_descriptor(&schema).unwrap();
    assert_eq!(descriptor.field.len(), 1);
    assert_eq!(descriptor.field[0].name, Some("active".to_string()));
    assert_eq!(descriptor.field[0].r#type, Some(Type::Bool as i32));
}

#[test]
fn test_generate_descriptor_float_types() {
    let schema = Schema::new(vec![
        Field::new("float32", DataType::Float32, false),
        Field::new("float64", DataType::Float64, false),
    ]);

    let descriptor = conversion::generate_protobuf_descriptor(&schema).unwrap();
    assert_eq!(descriptor.field.len(), 2);
    assert_eq!(descriptor.field[0].r#type, Some(Type::Float as i32));
    assert_eq!(descriptor.field[1].r#type, Some(Type::Double as i32));
}

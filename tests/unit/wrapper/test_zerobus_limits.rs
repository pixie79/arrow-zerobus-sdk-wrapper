//! Unit tests for Zerobus limits compliance
//!
//! Tests for:
//! - 4MB record size limit
//! - ASCII-only name validation
//! - Column count limit (2000)

use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::{
    WrapperConfiguration,
    error::ZerobusError,
    wrapper::conversion,
};
use prost_types::{
    field_descriptor_proto::{Label, Type},
    DescriptorProto, FieldDescriptorProto,
};
use std::sync::Arc;

#[test]
fn test_record_size_limit_exceeded() {
    // Create a record that exceeds 4MB limit
    // Zerobus limit: 4,194,285 bytes (4MB - 19 bytes for headers)
    let large_string = "x".repeat(4_200_000); // Exceeds limit
    
    let schema = Schema::new(vec![Field::new("large_field", DataType::Utf8, false)]);
    let string_array = StringArray::from(vec![large_string]);
    
    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![Arc::new(string_array)],
    ).unwrap();
    
    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("large_field".to_string()),
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
    // Should have 1 failed row due to size limit
    assert_eq!(result.failed_rows.len(), 1);
    assert_eq!(result.successful_bytes.len(), 0);
    
    if let Some((_, error)) = result.failed_rows.first() {
        if let ZerobusError::ConversionError(msg) = error {
            assert!(
                msg.contains("exceeds Zerobus limit"),
                "Error message should mention Zerobus limit: {}",
                msg
            );
            assert!(
                msg.contains("4194285"),
                "Error message should mention the limit: {}",
                msg
            );
        } else {
            panic!("Expected ConversionError, got: {:?}", error);
        }
    }
}

#[test]
fn test_record_size_at_limit() {
    // Create a record exactly at the 4MB limit
    let large_string = "x".repeat(4_194_280); // Just under limit (accounting for encoding overhead)
    
    let schema = Schema::new(vec![Field::new("large_field", DataType::Utf8, false)]);
    let string_array = StringArray::from(vec![large_string]);
    
    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![Arc::new(string_array)],
    ).unwrap();
    
    let descriptor = DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("large_field".to_string()),
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
    // Should succeed if within limit
    assert_eq!(result.failed_rows.len(), 0);
    assert_eq!(result.successful_bytes.len(), 1);
}

#[test]
fn test_table_name_ascii_only_valid() {
    // Valid table names: ASCII letters, digits, underscores
    // Supports Unity Catalog format: catalog.schema.table, schema.table, or table
    let valid_names = vec![
        "table1",                    // Simple table name
        "my_table",                  // Table with underscore
        "Table123",                  // Mixed case
        "TABLE_NAME",                // Uppercase
        "a1b2c3",                    // Alphanumeric
        "schema.table",              // 2-part name (schema.table)
        "catalog.schema.table",      // 3-part name (catalog.schema.table)
        "my_catalog.my_schema.my_table", // Full qualified name
        "catalog1.schema_2.table_3", // With numbers
    ];
    
    for name in valid_names {
        let config = WrapperConfiguration::new(
            "https://test.cloud.databricks.com".to_string(),
            name.to_string(),
        );
        assert!(
            config.validate().is_ok(),
            "Table name '{}' should be valid",
            name
        );
    }
}

#[test]
fn test_table_name_ascii_only_invalid() {
    // Invalid table names: non-ASCII characters, special chars (except underscore and dot as separator)
    let invalid_names = vec![
        ("table-name", "hyphen"),                    // hyphen in part
        ("table name", "space"),                     // space
        ("table@name", "@ symbol"),                  // @ symbol
        ("table#name", "hash"),                      // hash
        ("café", "non-ASCII"),                       // non-ASCII
        ("表", "non-ASCII"),                         // non-ASCII
        (".table", "leading dot"),                   // leading dot
        ("table.", "trailing dot"),                  // trailing dot
        ("schema..table", "double dot"),             // double dot
        ("catalog.schema.table.extra", "too many parts"), // 4 parts (max 3)
        ("", "empty"),                               // empty
    ];
    
    for (name, reason) in invalid_names {
        let config = WrapperConfiguration::new(
            "https://test.cloud.databricks.com".to_string(),
            name.to_string(),
        );
        let result = config.validate();
        assert!(
            result.is_err(),
            "Table name '{}' should be invalid (reason: {})",
            name,
            reason
        );
        
        if let Err(ZerobusError::ConfigurationError(msg)) = result {
            assert!(
                msg.contains("ASCII letters, digits, and underscores") 
                || msg.contains("cannot be empty")
                || msg.contains("format 'table'")
                || msg.contains("too many parts"),
                "Error message should mention validation requirement: {}",
                msg
            );
        } else {
            panic!("Expected ConfigurationError, got: {:?}", result);
        }
    }
}

#[test]
fn test_column_name_ascii_only_valid() {
    // Valid column names: ASCII letters, digits, underscores
    let schema = Schema::new(vec![
        Field::new("col1", DataType::Int32, false),
        Field::new("my_column", DataType::Int32, false),
        Field::new("Column123", DataType::Int32, false),
        Field::new("COLUMN_NAME", DataType::Int32, false),
    ]);
    
    let result = conversion::generate_protobuf_descriptor(&schema);
    assert!(result.is_ok(), "Valid column names should be accepted");
}

#[test]
fn test_column_name_ascii_only_invalid() {
    // Invalid column names: non-ASCII characters, special chars (except underscore)
    let invalid_schemas = vec![
        Schema::new(vec![Field::new("col-name", DataType::Int32, false)]),      // hyphen
        Schema::new(vec![Field::new("col.name", DataType::Int32, false)]),      // dot
        Schema::new(vec![Field::new("col name", DataType::Int32, false)]),      // space
        Schema::new(vec![Field::new("café", DataType::Int32, false)]),          // non-ASCII
    ];
    
    for schema in invalid_schemas {
        let result = conversion::generate_protobuf_descriptor(&schema);
        assert!(
            result.is_err(),
            "Schema with invalid column name should be rejected"
        );
        
        if let Err(ZerobusError::ConfigurationError(msg)) = result {
            assert!(
                msg.contains("ASCII letters, digits, and underscores"),
                "Error message should mention ASCII requirement: {}",
                msg
            );
        } else {
            panic!("Expected ConfigurationError, got: {:?}", result);
        }
    }
}

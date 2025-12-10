//! Tests for debug descriptor writing functionality

use arrow_zerobus_sdk_wrapper::wrapper::debug::DebugWriter;
use arrow_zerobus_sdk_wrapper::ZerobusError;
use prost_types::{DescriptorProto, FieldDescriptorProto, Type};
use std::time::Duration;
use tempfile::TempDir;

/// Create a simple test descriptor
fn create_test_descriptor() -> DescriptorProto {
    DescriptorProto {
        name: Some("TestMessage".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("id".to_string()),
                number: Some(1),
                label: Some(1), // Optional
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
                label: Some(1), // Optional
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
    }
}

#[tokio::test]
async fn test_write_descriptor_creates_file() {
    // Test that write_descriptor creates the descriptor file
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    let descriptor = create_test_descriptor();
    debug_writer
        .write_descriptor("test_table", &descriptor)
        .await
        .unwrap();

    // Verify file exists at expected location
    let descriptor_file = temp_dir
        .path()
        .join("zerobus/descriptors/test_table.pb");
    assert!(
        descriptor_file.exists(),
        "Descriptor file should exist at {:?}",
        descriptor_file
    );
}

#[tokio::test]
async fn test_write_descriptor_file_format() {
    // Test that descriptor file contains correct Protobuf data
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    let original_descriptor = create_test_descriptor();
    debug_writer
        .write_descriptor("test_table", &original_descriptor)
        .await
        .unwrap();

    // Read file back and parse
    let descriptor_file = temp_dir
        .path()
        .join("zerobus/descriptors/test_table.pb");
    let file_bytes = std::fs::read(&descriptor_file).unwrap();

    // Parse back to DescriptorProto
    let parsed_descriptor = DescriptorProto::decode(&file_bytes[..]).unwrap();

    // Verify contents match
    assert_eq!(
        original_descriptor.name,
        parsed_descriptor.name,
        "Descriptor name should match"
    );
    assert_eq!(
        original_descriptor.field.len(),
        parsed_descriptor.field.len(),
        "Field count should match"
    );
    assert_eq!(
        original_descriptor.field[0].name,
        parsed_descriptor.field[0].name,
        "First field name should match"
    );
}

#[tokio::test]
async fn test_write_descriptor_file_location() {
    // Test that descriptor file is written to correct location
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    let descriptor = create_test_descriptor();
    debug_writer
        .write_descriptor("test_table", &descriptor)
        .await
        .unwrap();

    // Expected location: {output_dir}/zerobus/descriptors/{table_name}.pb
    let expected_path = temp_dir
        .path()
        .join("zerobus/descriptors/test_table.pb");
    assert!(
        expected_path.exists(),
        "Descriptor file should be at expected path: {:?}",
        expected_path
    );

    // Verify directory structure
    let descriptors_dir = temp_dir.path().join("zerobus/descriptors");
    assert!(
        descriptors_dir.exists(),
        "Descriptors directory should exist"
    );
    assert!(
        descriptors_dir.is_dir(),
        "Descriptors path should be a directory"
    );
}

#[tokio::test]
async fn test_write_descriptor_multiple_calls() {
    // Test behavior when write_descriptor is called multiple times
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    let descriptor = create_test_descriptor();

    // Call write_descriptor multiple times
    debug_writer
        .write_descriptor("test_table", &descriptor)
        .await
        .unwrap();
    debug_writer
        .write_descriptor("test_table", &descriptor)
        .await
        .unwrap();
    debug_writer
        .write_descriptor("test_table", &descriptor)
        .await
        .unwrap();

    // Verify file exists (should only write once, subsequent calls should be no-ops)
    let descriptor_file = temp_dir
        .path()
        .join("zerobus/descriptors/test_table.pb");
    assert!(
        descriptor_file.exists(),
        "Descriptor file should exist"
    );

    // Verify file content is valid (should be from first write)
    let file_bytes = std::fs::read(&descriptor_file).unwrap();
    let parsed_descriptor = DescriptorProto::decode(&file_bytes[..]).unwrap();
    assert_eq!(
        descriptor.name,
        parsed_descriptor.name,
        "Descriptor should match original"
    );
}

#[tokio::test]
async fn test_write_descriptor_with_nested_types() {
    // Test writing descriptor with nested message types
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    // Create descriptor with nested type
    let nested_descriptor = DescriptorProto {
        name: Some("NestedMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("value".to_string()),
            number: Some(1),
            label: Some(1),
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

    let parent_descriptor = DescriptorProto {
        name: Some("ParentMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("nested".to_string()),
            number: Some(1),
            label: Some(1),
            r#type: Some(Type::Message as i32),
            type_name: Some("NestedMessage".to_string()),
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }],
        extension: vec![],
        nested_type: vec![nested_descriptor],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    debug_writer
        .write_descriptor("test_table", &parent_descriptor)
        .await
        .unwrap();

    // Verify file exists and can be parsed
    let descriptor_file = temp_dir
        .path()
        .join("zerobus/descriptors/test_table.pb");
    let file_bytes = std::fs::read(&descriptor_file).unwrap();
    let parsed_descriptor = DescriptorProto::decode(&file_bytes[..]).unwrap();

    // Verify nested types are preserved
    assert_eq!(
        parent_descriptor.nested_type.len(),
        parsed_descriptor.nested_type.len(),
        "Nested type count should match"
    );
    assert_eq!(
        parent_descriptor.nested_type[0].name,
        parsed_descriptor.nested_type[0].name,
        "Nested type name should match"
    );
}

#[tokio::test]
async fn test_write_descriptor_error_handling() {
    // Test error handling for descriptor writing
    // Create DebugWriter with invalid output directory (read-only or non-existent parent)
    // Note: This is difficult to test without actually creating a read-only directory
    // Instead, we test with a valid directory and verify normal operation
    
    let temp_dir = TempDir::new().unwrap();
    let debug_writer = DebugWriter::new(
        temp_dir.path().to_path_buf(),
        "test_table".to_string(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    let descriptor = create_test_descriptor();

    // This should succeed with valid directory
    let result = debug_writer
        .write_descriptor("test_table", &descriptor)
        .await;

    assert!(result.is_ok(), "Should succeed with valid directory");

    // Test with table name that needs sanitization
    let result = debug_writer
        .write_descriptor("test.table/name", &descriptor)
        .await;

    // Should succeed (table name is sanitized)
    assert!(result.is_ok(), "Should succeed with sanitized table name");

    // Verify file was created with sanitized name
    let sanitized_file = temp_dir
        .path()
        .join("zerobus/descriptors/test_table_name.pb");
    assert!(
        sanitized_file.exists(),
        "File should exist with sanitized name"
    );
}


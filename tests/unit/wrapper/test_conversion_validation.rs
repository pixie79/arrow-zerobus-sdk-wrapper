//! Unit tests for Protobuf descriptor validation
//!
//! Tests for validate_protobuf_descriptor function to ensure security
//! and prevent malicious or malformed descriptors

use arrow_zerobus_sdk_wrapper::wrapper::conversion;
use arrow_zerobus_sdk_wrapper::ZerobusError;
use prost_types::{
    field_descriptor_proto::{Label, Type},
    DescriptorProto, FieldDescriptorProto,
};

fn create_valid_descriptor() -> DescriptorProto {
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
fn test_validate_descriptor_valid_cases() {
    // Test that valid descriptors are accepted
    let descriptor = create_valid_descriptor();
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(result.is_ok(), "Valid descriptor should be accepted");
}

#[test]
fn test_validate_descriptor_max_nesting_depth() {
    // Create descriptor with 11 levels of nesting (exceeds max of 10)
    let mut descriptor = create_valid_descriptor();
    
    // Build 11 levels of nested types
    let mut current = &mut descriptor;
    for depth in 0..11 {
        let nested = DescriptorProto {
            name: Some(format!("NestedLevel{}", depth)),
            field: vec![],
            extension: vec![],
            nested_type: vec![],
            enum_type: vec![],
            extension_range: vec![],
            oneof_decl: vec![],
            options: None,
            reserved_range: vec![],
            reserved_name: vec![],
        };
        current.nested_type.push(nested);
        if let Some(last) = current.nested_type.last_mut() {
            current = last;
        }
    }
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_err(),
        "Descriptor with 11 levels of nesting should be rejected"
    );
    
    if let Err(ZerobusError::ConfigurationError(msg)) = result {
        assert!(
            msg.contains("nesting depth"),
            "Error message should mention nesting depth: {}",
            msg
        );
    } else {
        panic!("Expected ConfigurationError, got: {:?}", result);
    }
}

#[test]
fn test_validate_descriptor_max_fields() {
    // Create descriptor with 1001 fields (exceeds max of 1000)
    let mut descriptor = create_valid_descriptor();
    
    // Add 1001 fields
    descriptor.field.clear();
    for i in 1..=1001 {
        descriptor.field.push(FieldDescriptorProto {
            name: Some(format!("field_{}", i)),
            number: Some(i),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::Int32 as i32),
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        });
    }
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_err(),
        "Descriptor with 1001 fields should be rejected"
    );
    
    if let Err(ZerobusError::ConfigurationError(msg)) = result {
        assert!(
            msg.contains("field count"),
            "Error message should mention field count: {}",
            msg
        );
    } else {
        panic!("Expected ConfigurationError, got: {:?}", result);
    }
}

#[test]
fn test_validate_descriptor_invalid_field_number_too_low() {
    // Test field number < 1 (invalid)
    let mut descriptor = create_valid_descriptor();
    descriptor.field.push(FieldDescriptorProto {
        name: Some("invalid_field".to_string()),
        number: Some(0), // Invalid: must be >= 1
        label: Some(Label::Optional as i32),
        r#type: Some(Type::Int32 as i32),
        type_name: None,
        extendee: None,
        default_value: None,
        oneof_index: None,
        json_name: None,
        options: None,
        proto3_optional: None,
    });
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_err(),
        "Descriptor with field number 0 should be rejected"
    );
    
    if let Err(ZerobusError::ConfigurationError(msg)) = result {
        assert!(
            msg.contains("field number"),
            "Error message should mention field number: {}",
            msg
        );
    } else {
        panic!("Expected ConfigurationError, got: {:?}", result);
    }
}

#[test]
fn test_validate_descriptor_invalid_field_number_too_high() {
    // Test field number > 536870911 (invalid)
    let mut descriptor = create_valid_descriptor();
    descriptor.field.push(FieldDescriptorProto {
        name: Some("invalid_field".to_string()),
        number: Some(536870912), // Invalid: exceeds max
        label: Some(Label::Optional as i32),
        r#type: Some(Type::Int32 as i32),
        type_name: None,
        extendee: None,
        default_value: None,
        oneof_index: None,
        json_name: None,
        options: None,
        proto3_optional: None,
    });
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_err(),
        "Descriptor with field number 536870912 should be rejected"
    );
    
    if let Err(ZerobusError::ConfigurationError(msg)) = result {
        assert!(
            msg.contains("field number"),
            "Error message should mention field number: {}",
            msg
        );
    } else {
        panic!("Expected ConfigurationError, got: {:?}", result);
    }
}

#[test]
fn test_validate_descriptor_valid_field_numbers() {
    // Test valid field numbers at boundaries
    let mut descriptor = create_valid_descriptor();
    
    // Test minimum valid field number (1)
    descriptor.field.push(FieldDescriptorProto {
        name: Some("min_field".to_string()),
        number: Some(1),
        label: Some(Label::Optional as i32),
        r#type: Some(Type::Int32 as i32),
        type_name: None,
        extendee: None,
        default_value: None,
        oneof_index: None,
        json_name: None,
        options: None,
        proto3_optional: None,
    });
    
    // Test maximum valid field number (536870911)
    descriptor.field.push(FieldDescriptorProto {
        name: Some("max_field".to_string()),
        number: Some(536870911),
        label: Some(Label::Optional as i32),
        r#type: Some(Type::Int32 as i32),
        type_name: None,
        extendee: None,
        default_value: None,
        oneof_index: None,
        json_name: None,
        options: None,
        proto3_optional: None,
    });
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_ok(),
        "Descriptor with valid field numbers (1 and 536870911) should be accepted"
    );
}

#[test]
fn test_validate_descriptor_nested_validation() {
    // Test that nested types are also validated
    let mut descriptor = create_valid_descriptor();
    
    // Add a nested type with invalid field number
    let nested = DescriptorProto {
        name: Some("NestedMessage".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("invalid_nested_field".to_string()),
            number: Some(0), // Invalid
            label: Some(Label::Optional as i32),
            r#type: Some(Type::Int32 as i32),
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
    
    descriptor.nested_type.push(nested);
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_err(),
        "Nested type with invalid field number should be rejected"
    );
}

#[test]
fn test_validate_descriptor_deeply_nested_valid() {
    // Test that valid deeply nested structures are accepted (up to max depth)
    let mut descriptor = create_valid_descriptor();
    
    // Build 10 levels of nested types (max allowed)
    let mut current = &mut descriptor;
    for depth in 0..10 {
        let nested = DescriptorProto {
            name: Some(format!("NestedLevel{}", depth)),
            field: vec![FieldDescriptorProto {
                name: Some(format!("field_{}", depth)),
                number: Some(1),
                label: Some(Label::Optional as i32),
                r#type: Some(Type::Int32 as i32),
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
        current.nested_type.push(nested);
        if let Some(last) = current.nested_type.last_mut() {
            current = last;
        }
    }
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_ok(),
        "Descriptor with 10 levels of nesting (max allowed) should be accepted"
    );
}

#[test]
fn test_validate_descriptor_exactly_max_fields() {
    // Test descriptor with exactly 1000 fields (max allowed)
    let mut descriptor = create_valid_descriptor();
    
    descriptor.field.clear();
    for i in 1..=1000 {
        descriptor.field.push(FieldDescriptorProto {
            name: Some(format!("field_{}", i)),
            number: Some(i),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::Int32 as i32),
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        });
    }
    
    let result = conversion::validate_protobuf_descriptor(&descriptor);
    assert!(
        result.is_ok(),
        "Descriptor with exactly 1000 fields (max allowed) should be accepted"
    );
}


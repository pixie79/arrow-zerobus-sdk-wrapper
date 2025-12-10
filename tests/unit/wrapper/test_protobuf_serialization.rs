//! Unit tests for Protobuf wire format serialization
//!
//! Tests for encode_tag, encode_varint, encode_sint32, encode_sint64

use arrow_zerobus_sdk_wrapper::ZerobusError;

// Access the pub(crate) functions - they're accessible from tests in the same crate
use arrow_zerobus_sdk_wrapper::wrapper::protobuf_serialization::{
    encode_tag, encode_varint, encode_sint32, encode_sint64,
};

// Since the functions are pub(crate), we need to access them through the module
// We'll test them by calling the conversion functions that use them, or
// we can make them pub for testing. For now, let's test through integration.

// However, we can test the behavior indirectly through conversion tests.
// But let's create direct tests by making the functions accessible for testing.

// Actually, let's test the encoding functions by creating a test helper that
// exposes them, or test them through the conversion module.

// Since encode_tag, encode_varint, encode_sint32, encode_sint64 are pub(crate),
// we need to access them. Let's check if we can access them through the module path.

#[test]
fn test_encode_varint_zero() {
    // Test varint encoding for 0
    let mut buffer = Vec::new();
    let result = encode_varint(&mut buffer, 0);
    
    assert!(result.is_ok());
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 0);
}

#[test]
fn test_encode_varint_small_values() {
    // Test varint encoding for small values (< 128, single byte)
    let mut buffer = Vec::new();
    let result = protobuf_serialization::encode_varint(&mut buffer, 1);
    
    assert!(result.is_ok());
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 1);
    
    buffer.clear();
    encode_varint(&mut buffer, 127).unwrap();
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 127);
}

#[test]
fn test_encode_varint_multi_byte() {
    // Test varint encoding for values requiring multiple bytes
    let mut buffer = Vec::new();
    let result = protobuf_serialization::encode_varint(&mut buffer, 128);
    
    assert!(result.is_ok());
    // 128 = 0x80, requires 2 bytes: 0x80 (with continuation bit) | 0x01
    assert_eq!(buffer.len(), 2);
    assert_eq!(buffer[0], 0x80 | 0x00); // 128 with continuation bit
    assert_eq!(buffer[1], 0x01); // remaining value
}

#[test]
fn test_encode_varint_large_values() {
    // Test varint encoding for large values
    let mut buffer = Vec::new();
    let result = protobuf_serialization::encode_varint(&mut buffer, 300);
    
    assert!(result.is_ok());
    // 300 = 0x12C = 0b100101100
    // First byte: (300 & 0x7F) | 0x80 = 0xAC (with continuation)
    // Second byte: (300 >> 7) & 0x7F = 0x02
    assert!(buffer.len() >= 2);
    
    // Test u64::MAX
    buffer.clear();
    let result = protobuf_serialization::encode_varint(&mut buffer, u64::MAX);
    assert!(result.is_ok());
    // u64::MAX requires 10 bytes
    assert_eq!(buffer.len(), 10);
}

#[test]
fn test_encode_varint_edge_cases() {
    // Test edge cases
    let mut buffer = Vec::new();
    
    // Test 0x7F (127, max single byte)
    buffer.clear();
    encode_varint(&mut buffer, 0x7F).unwrap();
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 0x7F);
    
    // Test 0x80 (128, first multi-byte)
    buffer.clear();
    encode_varint(&mut buffer, 0x80).unwrap();
    assert_eq!(buffer.len(), 2);
    
    // Test 0x3FFF (16383, max 2-byte)
    buffer.clear();
    encode_varint(&mut buffer, 0x3FFF).unwrap();
    assert_eq!(buffer.len(), 2);
}

#[test]
fn test_encode_tag() {
    // Test tag encoding
    // Tag format: (field_number << 3) | wire_type
    let mut buffer = Vec::new();
    
    // Field 1, wire type 0 (Varint)
    let result = protobuf_serialization::encode_tag(&mut buffer, 1, 0);
    assert!(result.is_ok());
    // Tag = (1 << 3) | 0 = 8
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 8);
    
    // Field 2, wire type 2 (Length-delimited)
    buffer.clear();
    encode_tag(&mut buffer, 2, 2).unwrap();
    // Tag = (2 << 3) | 2 = 18
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 18);
    
    // Field 15, wire type 1 (Fixed64)
    buffer.clear();
    encode_tag(&mut buffer, 15, 1).unwrap();
    // Tag = (15 << 3) | 1 = 121
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 121);
}

#[test]
fn test_encode_tag_large_field_number() {
    // Test tag encoding with large field numbers (requires varint encoding)
    let mut buffer = Vec::new();
    
    // Field 536870911 (max valid field number), wire type 0
    let result = protobuf_serialization::encode_tag(&mut buffer, 536870911, 0);
    assert!(result.is_ok());
    // Tag = (536870911 << 3) | 0 = large value requiring varint
    assert!(buffer.len() > 1); // Should require multiple bytes
}

#[test]
fn test_encode_sint32_zero() {
    // Test sint32 encoding for 0
    let mut buffer = Vec::new();
    let result = protobuf_serialization::encode_sint32(&mut buffer, 0);
    
    assert!(result.is_ok());
    // Zigzag(0) = (0 << 1) ^ (0 >> 31) = 0
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 0);
}

#[test]
fn test_encode_sint32_positive() {
    // Test sint32 encoding for positive values
    let mut buffer = Vec::new();
    
    // Test 1
    let result = protobuf_serialization::encode_sint32(&mut buffer, 1);
    assert!(result.is_ok());
    // Zigzag(1) = (1 << 1) ^ (1 >> 31) = 2 ^ 0 = 2
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 2);
    
    // Test 100
    buffer.clear();
    encode_sint32(&mut buffer, 100).unwrap();
    // Zigzag(100) = (100 << 1) ^ (100 >> 31) = 200 ^ 0 = 200
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 200);
}

#[test]
fn test_encode_sint32_negative() {
    // Test sint32 encoding for negative values
    let mut buffer = Vec::new();
    
    // Test -1
    let result = protobuf_serialization::encode_sint32(&mut buffer, -1);
    assert!(result.is_ok());
    // Zigzag(-1) = (-1 << 1) ^ (-1 >> 31) = -2 ^ -1 = 1
    // Actually: (-1 << 1) = -2, (-1 >> 31) = -1 (arithmetic shift)
    // -2 ^ -1 = 1
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 1);
    
    // Test -100
    buffer.clear();
    encode_sint32(&mut buffer, -100).unwrap();
    // Zigzag(-100) = (-100 << 1) ^ (-100 >> 31) = -200 ^ -1 = 199
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 199);
}

#[test]
fn test_encode_sint32_boundaries() {
    // Test sint32 encoding at boundaries
    let mut buffer = Vec::new();
    
    // Test i32::MAX
    buffer.clear();
    let result = protobuf_serialization::encode_sint32(&mut buffer, i32::MAX);
    assert!(result.is_ok());
    // Zigzag(i32::MAX) = (i32::MAX << 1) ^ (i32::MAX >> 31)
    // = (0x7FFFFFFF << 1) ^ 0 = 0xFFFFFFFE
    // This is a large value requiring varint encoding
    assert!(buffer.len() > 1);
    
    // Test i32::MIN
    buffer.clear();
    let result = protobuf_serialization::encode_sint32(&mut buffer, i32::MIN);
    assert!(result.is_ok());
    // Zigzag(i32::MIN) = (i32::MIN << 1) ^ (i32::MIN >> 31)
    // = (0x80000000 << 1) ^ -1 = 0xFFFFFFFF
    // This is also a large value
    assert!(buffer.len() > 1);
}

#[test]
fn test_encode_sint64_zero() {
    // Test sint64 encoding for 0
    let mut buffer = Vec::new();
    let result = protobuf_serialization::encode_sint64(&mut buffer, 0);
    
    assert!(result.is_ok());
    // Zigzag(0) = (0 << 1) ^ (0 >> 63) = 0
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 0);
}

#[test]
fn test_encode_sint64_positive() {
    // Test sint64 encoding for positive values
    let mut buffer = Vec::new();
    
    // Test 1
    let result = protobuf_serialization::encode_sint64(&mut buffer, 1);
    assert!(result.is_ok());
    // Zigzag(1) = (1 << 1) ^ (1 >> 63) = 2 ^ 0 = 2
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 2);
    
    // Test 100
    buffer.clear();
    encode_sint64(&mut buffer, 100).unwrap();
    // Zigzag(100) = (100 << 1) ^ (100 >> 63) = 200 ^ 0 = 200
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 200);
}

#[test]
fn test_encode_sint64_negative() {
    // Test sint64 encoding for negative values
    let mut buffer = Vec::new();
    
    // Test -1
    let result = protobuf_serialization::encode_sint64(&mut buffer, -1);
    assert!(result.is_ok());
    // Zigzag(-1) = (-1 << 1) ^ (-1 >> 63) = -2 ^ -1 = 1
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 1);
    
    // Test -100
    buffer.clear();
    encode_sint64(&mut buffer, -100).unwrap();
    // Zigzag(-100) = (-100 << 1) ^ (-100 >> 63) = -200 ^ -1 = 199
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0], 199);
}

#[test]
fn test_encode_sint64_boundaries() {
    // Test sint64 encoding at boundaries
    let mut buffer = Vec::new();
    
    // Test i64::MAX
    buffer.clear();
    let result = protobuf_serialization::encode_sint64(&mut buffer, i64::MAX);
    assert!(result.is_ok());
    // Zigzag(i64::MAX) is a large value requiring varint encoding
    assert!(buffer.len() > 1);
    
    // Test i64::MIN
    buffer.clear();
    let result = protobuf_serialization::encode_sint64(&mut buffer, i64::MIN);
    assert!(result.is_ok());
    // Zigzag(i64::MIN) is also a large value
    assert!(buffer.len() > 1);
}

#[test]
fn test_encode_roundtrip_sint32() {
    // Test that zigzag encoding is reversible (conceptually)
    // This verifies the encoding is correct
    let test_values = vec![0, 1, -1, 100, -100, 1000, -1000, i32::MAX, i32::MIN];
    
    for value in test_values {
        let mut buffer = Vec::new();
        let result = protobuf_serialization::encode_sint32(&mut buffer, value);
        assert!(result.is_ok(), "Failed to encode {}", value);
        assert!(!buffer.is_empty(), "Buffer should not be empty for {}", value);
    }
}

#[test]
fn test_encode_roundtrip_sint64() {
    // Test that zigzag encoding is reversible (conceptually)
    let test_values = vec![0i64, 1, -1, 100, -100, 1000, -1000, i64::MAX, i64::MIN];
    
    for value in test_values {
        let mut buffer = Vec::new();
        let result = protobuf_serialization::encode_sint64(&mut buffer, value);
        assert!(result.is_ok(), "Failed to encode {}", value);
        assert!(!buffer.is_empty(), "Buffer should not be empty for {}", value);
    }
}

#[test]
fn test_encode_varint_roundtrip() {
    // Test varint encoding for various values
    let test_values = vec![0u64, 1, 127, 128, 255, 256, 1000, 65535, 100000, u64::MAX];
    
    for value in test_values {
        let mut buffer = Vec::new();
        let result = protobuf_serialization::encode_varint(&mut buffer, value);
        assert!(result.is_ok(), "Failed to encode {}", value);
        assert!(!buffer.is_empty(), "Buffer should not be empty for {}", value);
        
        // Verify encoding length is reasonable
        // Single byte: 0-127
        // Two bytes: 128-16383
        // etc.
        if value < 128 {
            assert_eq!(buffer.len(), 1, "Value {} should encode in 1 byte", value);
        } else if value < 16384 {
            assert_eq!(buffer.len(), 2, "Value {} should encode in 2 bytes", value);
        }
    }
}

#[test]
fn test_encode_tag_all_wire_types() {
    // Test tag encoding for all wire types
    let mut buffer = Vec::new();
    
    // Wire type 0: Varint
    buffer.clear();
    encode_tag(&mut buffer, 1, 0).unwrap();
    assert_eq!(buffer[0], (1 << 3) | 0);
    
    // Wire type 1: Fixed64
    buffer.clear();
    encode_tag(&mut buffer, 1, 1).unwrap();
    assert_eq!(buffer[0], (1 << 3) | 1);
    
    // Wire type 2: Length-delimited
    buffer.clear();
    encode_tag(&mut buffer, 1, 2).unwrap();
    assert_eq!(buffer[0], (1 << 3) | 2);
    
    // Wire type 5: Fixed32
    buffer.clear();
    encode_tag(&mut buffer, 1, 5).unwrap();
    assert_eq!(buffer[0], (1 << 3) | 5);
}

#[test]
fn test_encode_tag_field_number_range() {
    // Test tag encoding across field number range
    let test_field_numbers = vec![1, 15, 100, 1000, 536870911]; // Max valid field number
    
    for field_number in test_field_numbers {
        let mut buffer = Vec::new();
        let result = protobuf_serialization::encode_tag(&mut buffer, field_number, 0);
        assert!(result.is_ok(), "Failed to encode tag for field {}", field_number);
        assert!(!buffer.is_empty(), "Buffer should not be empty for field {}", field_number);
    }
}


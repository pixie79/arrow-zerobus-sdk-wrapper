//! Protobuf wire format serialization utilities
//!
//! This module provides low-level functions for encoding Protobuf wire format.
//! Reused from cap-gl-consumer-rust/src/writer/protobuf_serialization.rs

use crate::error::ZerobusError;

/// Encode a Protobuf field tag
///
/// Tag format: (field_number << 3) | wire_type
///
/// # Arguments
///
/// * `buffer` - Buffer to write tag to
/// * `field_number` - Protobuf field number
/// * `wire_type` - Protobuf wire type (0=Varint, 1=Fixed64, 2=Length-delimited, 5=Fixed32)
pub(crate) fn encode_tag(
    buffer: &mut Vec<u8>,
    field_number: i32,
    wire_type: u32,
) -> Result<(), ZerobusError> {
    let tag = ((field_number as u32) << 3) | wire_type;
    encode_varint(buffer, tag as u64)
}

/// Encode varint (variable-length integer)
///
/// Protobuf uses varint encoding for integers and tags.
/// Each byte has 7 bits of data and 1 continuation bit.
///
/// # Arguments
///
/// * `buffer` - Buffer to write varint to
/// * `value` - Value to encode as varint
pub(crate) fn encode_varint(buffer: &mut Vec<u8>, mut value: u64) -> Result<(), ZerobusError> {
    while value >= 0x80 {
        buffer.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
    buffer.push((value & 0x7F) as u8);
    Ok(())
}

/// Encode signed integer using zigzag encoding
///
/// Zigzag encoding converts signed integers to unsigned integers for efficient encoding.
/// Formula: (n << 1) ^ (n >> 31) for 32-bit, (n << 1) ^ (n >> 63) for 64-bit
///
/// # Arguments
///
/// * `buffer` - Buffer to write encoded value to
/// * `value` - Signed integer value to encode
pub(crate) fn encode_sint32(buffer: &mut Vec<u8>, value: i32) -> Result<(), ZerobusError> {
    // Zigzag encoding: (n << 1) ^ (n >> 31)
    let zigzag = ((value << 1) ^ (value >> 31)) as u32;
    encode_varint(buffer, zigzag as u64)
}

/// Encode signed 64-bit integer using zigzag encoding
///
/// Zigzag encoding converts signed integers to unsigned integers for efficient encoding.
/// Formula: (n << 1) ^ (n >> 63)
///
/// # Arguments
///
/// * `buffer` - Buffer to write encoded value to
/// * `value` - Signed 64-bit integer value to encode
pub(crate) fn encode_sint64(buffer: &mut Vec<u8>, value: i64) -> Result<(), ZerobusError> {
    // Zigzag encoding: (n << 1) ^ (n >> 63)
    let zigzag = ((value << 1) ^ (value >> 63)) as u64;
    encode_varint(buffer, zigzag)
}

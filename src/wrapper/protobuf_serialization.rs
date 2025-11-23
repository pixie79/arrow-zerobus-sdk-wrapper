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
pub(crate) fn encode_tag(buffer: &mut Vec<u8>, field_number: i32, wire_type: u32) -> Result<(), ZerobusError> {
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


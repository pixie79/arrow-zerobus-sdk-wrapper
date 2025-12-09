//! Arrow to Protobuf conversion
//!
//! This module handles conversion of Arrow RecordBatch data to Protobuf format
//! required by Zerobus. Reuses conversion logic from cap-gl-consumer-rust.

use crate::error::ZerobusError;
use crate::wrapper::protobuf_serialization::{encode_tag, encode_varint};
use arrow::array::*;
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use prost_types::{
    field_descriptor_proto::Label, field_descriptor_proto::Type, DescriptorProto,
    FieldDescriptorProto,
};
use std::sync::Arc;
use tracing::debug;

/// Result of converting a RecordBatch to Protobuf
#[derive(Debug)]
pub struct ProtobufConversionResult {
    /// Successful conversions: (row_index, protobuf_bytes)
    pub successful_bytes: Vec<(usize, Vec<u8>)>,
    /// Failed conversions: (row_index, error_message)
    pub failed_rows: Vec<(usize, String)>,
}

/// Convert Arrow RecordBatch to Protobuf bytes
///
/// Converts each row in the RecordBatch to Protobuf bytes using the descriptor.
/// Returns both successful conversions and failed rows.
///
/// # Arguments
///
/// * `batch` - RecordBatch to convert
/// * `descriptor` - Protobuf descriptor that matches the batch schema
///
/// # Returns
///
/// Returns ProtobufConversionResult with successful bytes and failed rows.
///
/// # Errors
///
/// Returns `ConversionError` if conversion fails completely.
pub fn record_batch_to_protobuf_bytes(
    batch: &RecordBatch,
    descriptor: &DescriptorProto,
) -> Result<Vec<Vec<u8>>, ZerobusError> {
    let schema = batch.schema();
    let num_rows = batch.num_rows();

    if num_rows == 0 {
        return Ok(vec![]);
    }

    // Build field name -> field descriptor map for efficient lookup
    let field_by_name: std::collections::HashMap<String, &FieldDescriptorProto> = descriptor
        .field
        .iter()
        .filter_map(|f| f.name.as_ref().map(|name| (name.clone(), f)))
        .collect();

    // Build nested type name -> nested descriptor map
    let nested_types_by_name: std::collections::HashMap<String, &DescriptorProto> = descriptor
        .nested_type
        .iter()
        .filter_map(|nt| {
            nt.name.as_ref().map(|name| {
                // Extract the full type name (e.g., ".ZerobusMessage._metadata" -> "_metadata")
                // The type_name in FieldDescriptorProto uses format ".ParentMessage.NestedMessage"
                // We need to match on the nested message name
                (name.clone(), nt)
            })
        })
        .collect();

    let mut protobuf_bytes_list = Vec::new();

    // Convert each row directly from Arrow to Protobuf
    for row_idx in 0..num_rows {
        let mut row_buffer = Vec::new();

        // Encode each field directly from Arrow array to Protobuf wire format
        for (field_idx, field) in schema.fields().iter().enumerate() {
            let array = batch.column(field_idx);

            // Find field descriptor
            if let Some(field_desc) = field_by_name.get(field.name()) {
                let field_number = field_desc.number.unwrap_or(0);

                if let Err(e) = encode_arrow_field_to_protobuf(
                    &mut row_buffer,
                    field_number,
                    field_desc,
                    array,
                    row_idx,
                    descriptor,
                    Some(&nested_types_by_name),
                ) {
                    return Err(ZerobusError::ConversionError(format!(
                        "Failed to encode field '{}' at row {}: {}",
                        field.name(),
                        row_idx,
                        e
                    )));
                }
            } else {
                debug!("Field '{}' not found in descriptor, skipping", field.name());
            }
        }

        protobuf_bytes_list.push(row_buffer);
    }

    Ok(protobuf_bytes_list)
}

/// Encode a field value from Arrow array directly to Protobuf wire format
///
/// This preserves type precision (Int64 vs Int32, Float64 vs Float32, etc.)
/// by converting directly from Arrow types to Protobuf wire format.
///
/// # Arguments
///
/// * `buffer` - Buffer to write Protobuf bytes to
/// * `field_number` - Protobuf field number
/// * `field_desc` - Protobuf field descriptor
/// * `array` - Arrow array containing the field values
/// * `row_idx` - Row index to extract value from
/// * `parent_descriptor` - Parent message descriptor (for nested types)
/// * `nested_types` - Optional map of nested type names to descriptors
fn encode_arrow_field_to_protobuf(
    buffer: &mut Vec<u8>,
    field_number: i32,
    field_desc: &FieldDescriptorProto,
    array: &Arc<dyn Array>,
    row_idx: usize,
    _parent_descriptor: &DescriptorProto,
    nested_types: Option<&std::collections::HashMap<String, &DescriptorProto>>,
) -> Result<(), ZerobusError> {
    if array.is_null(row_idx) {
        // Protobuf doesn't encode null/optional fields - just skip
        return Ok(());
    }

    let protobuf_type = field_desc.r#type.unwrap_or(9); // Default to String
    let is_repeated = field_desc.label == Some(Label::Repeated as i32);

    // Handle repeated fields
    // CRITICAL: Check repeated FIRST, even for nested messages (repeated nested messages are ListArray of StructArray)
    if is_repeated {
        if let Some(list_array) = array.as_any().downcast_ref::<ListArray>() {
            let offsets = list_array.value_offsets();
            let start = offsets[row_idx] as usize;
            let end = offsets[row_idx + 1] as usize;
            let values = list_array.values();

            // For repeated nested messages (type 11), each element in the list is a StructArray
            if protobuf_type == 11 {
                // Repeated nested message - encode each StructArray element as a nested message
                // Find the nested type descriptor
                if let Some(type_name) = &field_desc.type_name {
                    // Extract nested message name from type_name
                    let nested_descriptor = if let Some(nested_map) = nested_types {
                        let parts: Vec<&str> =
                            type_name.trim_start_matches('.').split('.').collect();
                        if let Some(last_part) = parts.last() {
                            nested_map.get(*last_part)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(nested_desc) = nested_descriptor {
                        // Verify values is a StructArray
                        if let Some(struct_array) = values.as_any().downcast_ref::<StructArray>() {
                            // Encode each element in the list as a nested message
                            for i in start..end {
                                if !struct_array.is_null(i) {
                                    // Encode as length-delimited (wire type 2)
                                    let wire_type = 2u32;
                                    encode_tag(buffer, field_number, wire_type)?;

                                    // Encode nested message fields
                                    let mut nested_buffer = Vec::new();
                                    let nested_schema = struct_array.fields();

                                    // Build field name -> field descriptor map for nested message
                                    let nested_field_by_name: std::collections::HashMap<
                                        String,
                                        &FieldDescriptorProto,
                                    > = nested_desc
                                        .field
                                        .iter()
                                        .filter_map(|f| {
                                            f.name.as_ref().map(|name| (name.clone(), f))
                                        })
                                        .collect();

                                    // Recursively build nested types map for nested message
                                    let nested_nested_types: std::collections::HashMap<
                                        String,
                                        &DescriptorProto,
                                    > = nested_desc
                                        .nested_type
                                        .iter()
                                        .filter_map(|nt| {
                                            nt.name.as_ref().map(|name| (name.clone(), nt))
                                        })
                                        .collect();

                                    // Encode each field in the nested struct
                                    for (field_idx, field) in nested_schema.iter().enumerate() {
                                        let nested_array = struct_array.column(field_idx);

                                        if let Some(nested_field_desc) =
                                            nested_field_by_name.get(field.name())
                                        {
                                            let nested_field_number =
                                                nested_field_desc.number.unwrap_or(0);

                                            if let Err(e) = encode_arrow_field_to_protobuf(
                                                &mut nested_buffer,
                                                nested_field_number,
                                                nested_field_desc,
                                                nested_array,
                                                i, // Use list element index, not row_idx
                                                nested_desc,
                                                Some(&nested_nested_types),
                                            ) {
                                                return Err(ZerobusError::ConversionError(format!(
                                                    "Failed to encode element {} of repeated nested message '{}': {}",
                                                    i,
                                                    field_desc.name.as_ref().unwrap_or(&"unknown".to_string()),
                                                    e
                                                )));
                                            }
                                        }
                                    }

                                    // Write length-delimited nested message
                                    encode_varint(buffer, nested_buffer.len() as u64)?;
                                    buffer.extend_from_slice(&nested_buffer);
                                }
                            }
                            return Ok(());
                        } else {
                            return Err(ZerobusError::ConversionError(format!(
                                "Repeated nested message '{}' values array is not a StructArray",
                                field_desc.name.as_ref().unwrap_or(&"unknown".to_string())
                            )));
                        }
                    } else {
                        return Err(ZerobusError::ConversionError(format!(
                            "Nested type descriptor not found for repeated nested message field '{}' with type_name '{}'",
                            field_desc.name.as_ref().unwrap_or(&"unknown".to_string()),
                            type_name
                        )));
                    }
                } else {
                    return Err(ZerobusError::ConversionError(format!(
                        "Repeated nested message field '{}' missing type_name",
                        field_desc.name.as_ref().unwrap_or(&"unknown".to_string())
                    )));
                }
            } else {
                // Repeated primitive or other type - encode each element
                for i in start..end {
                    if !values.is_null(i) {
                        encode_arrow_value_to_protobuf(
                            buffer,
                            field_number,
                            field_desc,
                            values,
                            i,
                        )?;
                    }
                }
                return Ok(());
            }
        } else if protobuf_type == 11 {
            // Field is marked as repeated and type 11 (Message), but array is not ListArray
            // This can happen if the Arrow schema generation created a different structure
            // Try to handle it as a single nested message (fallback for edge cases)
            // This will be handled by the single nested message code below
        }
    }

    // Handle single nested messages (type 11 = Message)
    if protobuf_type == 11 {
        // Find the nested type descriptor
        if let Some(type_name) = &field_desc.type_name {
            // Extract nested message name from type_name (format: ".ParentMessage.NestedMessage")
            // We need to find the nested descriptor
            let nested_descriptor = if let Some(nested_map) = nested_types {
                // Extract the nested message name from type_name
                // type_name format: ".ZerobusMessage.ZerobusMessage_FieldName" -> look for "ZerobusMessage_FieldName"
                // The nested type name is the last part after splitting by "."
                let parts: Vec<&str> = type_name.trim_start_matches('.').split('.').collect();
                if let Some(last_part) = parts.last() {
                    nested_map.get(*last_part)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(nested_desc) = nested_descriptor {
                // Encode nested message
                if let Some(struct_array) = array.as_any().downcast_ref::<StructArray>() {
                    // Encode as length-delimited (wire type 2)
                    let wire_type = 2u32;
                    encode_tag(buffer, field_number, wire_type)?;

                    // Encode nested message fields
                    let mut nested_buffer = Vec::new();
                    let nested_schema = struct_array.fields();

                    // Build field name -> field descriptor map for nested message
                    let nested_field_by_name: std::collections::HashMap<
                        String,
                        &FieldDescriptorProto,
                    > = nested_desc
                        .field
                        .iter()
                        .filter_map(|f| f.name.as_ref().map(|name| (name.clone(), f)))
                        .collect();

                    // Recursively build nested types map for nested message
                    let nested_nested_types: std::collections::HashMap<String, &DescriptorProto> =
                        nested_desc
                            .nested_type
                            .iter()
                            .filter_map(|nt| nt.name.as_ref().map(|name| (name.clone(), nt)))
                            .collect();

                    // Encode each field in the nested struct
                    for (field_idx, field) in nested_schema.iter().enumerate() {
                        let nested_array = struct_array.column(field_idx);

                        if let Some(nested_field_desc) = nested_field_by_name.get(field.name()) {
                            let nested_field_number = nested_field_desc.number.unwrap_or(0);

                            if let Err(e) = encode_arrow_field_to_protobuf(
                                &mut nested_buffer,
                                nested_field_number,
                                nested_field_desc,
                                nested_array,
                                row_idx,
                                nested_desc,
                                Some(&nested_nested_types),
                            ) {
                                return Err(ZerobusError::ConversionError(format!(
                                    "Failed to encode nested field '{}' at row {}: {}",
                                    field.name(),
                                    row_idx,
                                    e
                                )));
                            }
                        }
                    }

                    // Write length-delimited nested message
                    encode_varint(buffer, nested_buffer.len() as u64)?;
                    buffer.extend_from_slice(&nested_buffer);
                    return Ok(());
                } else {
                    return Err(ZerobusError::ConversionError(format!(
                        "Expected StructArray for nested message field '{}'",
                        field_desc.name.as_ref().unwrap_or(&"unknown".to_string())
                    )));
                }
            } else {
                return Err(ZerobusError::ConversionError(format!(
                    "Nested type descriptor not found for field '{}' with type_name '{}'",
                    field_desc.name.as_ref().unwrap_or(&"unknown".to_string()),
                    type_name
                )));
            }
        } else {
            return Err(ZerobusError::ConversionError(format!(
                "Nested message field '{}' missing type_name",
                field_desc.name.as_ref().unwrap_or(&"unknown".to_string())
            )));
        }
    }

    // CRITICAL: Before falling through to encode_arrow_value_to_protobuf, check if this is actually a nested message
    // This handles cases where type 11 fields might not have been caught by the earlier checks
    // We dynamically detect nested messages by checking:
    // 1. If descriptor says type 11 (but wasn't handled above - shouldn't happen, but safety check)
    // 2. If array is StructArray (indicates nested message structure)
    // 3. If type_name is set (indicates this is a message type)

    // Safety check: If type is 11 but we reached here, something went wrong - handle it anyway
    if protobuf_type == 11 {
        // This is a nested message that wasn't handled above - encode it recursively
        // Find the nested type descriptor
        if let Some(type_name) = &field_desc.type_name {
            let nested_descriptor = if let Some(nested_map) = nested_types {
                let parts: Vec<&str> = type_name.trim_start_matches('.').split('.').collect();
                if let Some(last_part) = parts.last() {
                    nested_map.get(*last_part)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(nested_desc) = nested_descriptor {
                // Encode nested message
                if let Some(struct_array) = array.as_any().downcast_ref::<StructArray>() {
                    // Encode as length-delimited (wire type 2)
                    let wire_type = 2u32;
                    encode_tag(buffer, field_number, wire_type)?;

                    // Encode nested message fields
                    let mut nested_buffer = Vec::new();
                    let nested_schema = struct_array.fields();

                    // Build field name -> field descriptor map for nested message
                    let nested_field_by_name: std::collections::HashMap<
                        String,
                        &FieldDescriptorProto,
                    > = nested_desc
                        .field
                        .iter()
                        .filter_map(|f| f.name.as_ref().map(|name| (name.clone(), f)))
                        .collect();

                    // Recursively build nested types map for nested message
                    let nested_nested_types: std::collections::HashMap<String, &DescriptorProto> =
                        nested_desc
                            .nested_type
                            .iter()
                            .filter_map(|nt| nt.name.as_ref().map(|name| (name.clone(), nt)))
                            .collect();

                    // Encode each field in the nested struct
                    for (field_idx, field) in nested_schema.iter().enumerate() {
                        let nested_array = struct_array.column(field_idx);

                        if let Some(nested_field_desc) = nested_field_by_name.get(field.name()) {
                            let nested_field_number = nested_field_desc.number.unwrap_or(0);

                            if let Err(e) = encode_arrow_field_to_protobuf(
                                &mut nested_buffer,
                                nested_field_number,
                                nested_field_desc,
                                nested_array,
                                row_idx,
                                nested_desc,
                                Some(&nested_nested_types),
                            ) {
                                return Err(ZerobusError::ConversionError(format!(
                                    "Failed to encode nested field '{}' at row {}: {}",
                                    field.name(),
                                    row_idx,
                                    e
                                )));
                            }
                        }
                    }

                    // Write length-delimited nested message
                    encode_varint(buffer, nested_buffer.len() as u64)?;
                    buffer.extend_from_slice(&nested_buffer);
                    return Ok(());
                }
            }
        }
    }

    // Also check if array is StructArray - this indicates a nested message even if descriptor type is wrong
    if array.as_any().downcast_ref::<StructArray>().is_some() {
        // Array is StructArray but descriptor doesn't indicate nested message
        // This might be a nested message with incorrect descriptor - try to encode it
        if let Some(type_name) = &field_desc.type_name {
            let nested_descriptor = if let Some(nested_map) = nested_types {
                let parts: Vec<&str> = type_name.trim_start_matches('.').split('.').collect();
                if let Some(last_part) = parts.last() {
                    nested_map.get(*last_part)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(nested_desc) = nested_descriptor {
                if let Some(struct_array) = array.as_any().downcast_ref::<StructArray>() {
                    // Encode as length-delimited (wire type 2)
                    let wire_type = 2u32;
                    encode_tag(buffer, field_number, wire_type)?;

                    let mut nested_buffer = Vec::new();
                    let nested_schema = struct_array.fields();

                    let nested_field_by_name: std::collections::HashMap<
                        String,
                        &FieldDescriptorProto,
                    > = nested_desc
                        .field
                        .iter()
                        .filter_map(|f| f.name.as_ref().map(|name| (name.clone(), f)))
                        .collect();

                    let nested_nested_types: std::collections::HashMap<String, &DescriptorProto> =
                        nested_desc
                            .nested_type
                            .iter()
                            .filter_map(|nt| nt.name.as_ref().map(|name| (name.clone(), nt)))
                            .collect();

                    for (field_idx, field) in nested_schema.iter().enumerate() {
                        let nested_array = struct_array.column(field_idx);

                        if let Some(nested_field_desc) = nested_field_by_name.get(field.name()) {
                            let nested_field_number = nested_field_desc.number.unwrap_or(0);

                            if let Err(e) = encode_arrow_field_to_protobuf(
                                &mut nested_buffer,
                                nested_field_number,
                                nested_field_desc,
                                nested_array,
                                row_idx,
                                nested_desc,
                                Some(&nested_nested_types),
                            ) {
                                return Err(ZerobusError::ConversionError(format!(
                                    "Failed to encode nested field '{}' at row {}: {}",
                                    field.name(),
                                    row_idx,
                                    e
                                )));
                            }
                        }
                    }

                    encode_varint(buffer, nested_buffer.len() as u64)?;
                    buffer.extend_from_slice(&nested_buffer);
                    return Ok(());
                }
            }
        }
    }

    // Handle primitive types
    encode_arrow_value_to_protobuf(buffer, field_number, field_desc, array, row_idx)
}

/// Encode a single Arrow value to Protobuf wire format
fn encode_arrow_value_to_protobuf(
    buffer: &mut Vec<u8>,
    field_number: i32,
    field_desc: &FieldDescriptorProto,
    array: &Arc<dyn Array>,
    row_idx: usize,
) -> Result<(), ZerobusError> {
    let protobuf_type = field_desc.r#type.unwrap_or(9);

    match protobuf_type {
        1 => {
            // Double (Float64)
            let arr = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    ZerobusError::ConversionError("Expected Float64Array".to_string())
                })?;
            let wire_type = 1u32; // Fixed64
            encode_tag(buffer, field_number, wire_type)?;
            buffer.extend_from_slice(&arr.value(row_idx).to_le_bytes());
            Ok(())
        }
        2 => {
            // Float (Float32)
            let arr = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| {
                    ZerobusError::ConversionError("Expected Float32Array".to_string())
                })?;
            let wire_type = 5u32; // Fixed32
            encode_tag(buffer, field_number, wire_type)?;
            buffer.extend_from_slice(&arr.value(row_idx).to_le_bytes());
            Ok(())
        }
        3 => {
            // Int64
            // Handle both Int64Array and TimestampArray (which stores time as Int64 internally)
            if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                encode_varint(buffer, arr.value(row_idx) as u64)?;
                Ok(())
            } else if let Some(arr) = array
                .as_any()
                .downcast_ref::<arrow::array::TimestampMicrosecondArray>()
            {
                // TimestampArray stores microseconds as Int64 internally
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                encode_varint(buffer, arr.value(row_idx) as u64)?;
                Ok(())
            } else if let Some(arr) = array
                .as_any()
                .downcast_ref::<arrow::array::TimestampMillisecondArray>()
            {
                // TimestampArray stores milliseconds as Int64 internally, convert to microseconds
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                encode_varint(buffer, (arr.value(row_idx) * 1000) as u64)?; // Convert ms to μs
                Ok(())
            } else if let Some(arr) = array
                .as_any()
                .downcast_ref::<arrow::array::TimestampSecondArray>()
            {
                // TimestampArray stores seconds as Int64 internally, convert to microseconds
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                encode_varint(buffer, (arr.value(row_idx) * 1_000_000) as u64)?; // Convert s to μs
                Ok(())
            } else if let Some(arr) = array
                .as_any()
                .downcast_ref::<arrow::array::TimestampNanosecondArray>()
            {
                // TimestampArray stores nanoseconds as Int64 internally, convert to microseconds
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                encode_varint(buffer, (arr.value(row_idx) / 1000) as u64)?; // Convert ns to μs
                Ok(())
            } else {
                Err(ZerobusError::ConversionError(format!(
                    "Expected Int64Array or TimestampArray for Int64 field, got: {:?}",
                    array.data_type()
                )))
            }
        }
        4 => {
            // UInt64
            let arr = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected UInt64Array".to_string()))?;
            let wire_type = 0u32; // Varint
            encode_tag(buffer, field_number, wire_type)?;
            encode_varint(buffer, arr.value(row_idx))?;
            Ok(())
        }
        5 => {
            // Int32
            let arr = array
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected Int32Array".to_string()))?;
            let wire_type = 0u32; // Varint
            encode_tag(buffer, field_number, wire_type)?;
            encode_varint(buffer, arr.value(row_idx) as u64)?;
            Ok(())
        }
        8 => {
            // Bool
            let arr = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    ZerobusError::ConversionError("Expected BooleanArray".to_string())
                })?;
            let wire_type = 0u32; // Varint
            encode_tag(buffer, field_number, wire_type)?;
            encode_varint(buffer, if arr.value(row_idx) { 1 } else { 0 })?;
            Ok(())
        }
        9 => {
            // String
            let arr = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected StringArray".to_string()))?;
            let wire_type = 2u32; // Length-delimited
            encode_tag(buffer, field_number, wire_type)?;
            let bytes = arr.value(row_idx).as_bytes();
            encode_varint(buffer, bytes.len() as u64)?;
            buffer.extend_from_slice(bytes);
            Ok(())
        }
        12 => {
            // Bytes
            let arr = array
                .as_any()
                .downcast_ref::<BinaryArray>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected BinaryArray".to_string()))?;
            let wire_type = 2u32; // Length-delimited
            encode_tag(buffer, field_number, wire_type)?;
            let bytes = arr.value(row_idx);
            encode_varint(buffer, bytes.len() as u64)?;
            buffer.extend_from_slice(bytes);
            Ok(())
        }
        17 => {
            // SInt32 (signed int32 with zigzag encoding)
            // Often used for enum values
            // Handle case where Arrow has StringArray but descriptor says SInt32 (enum stored as string)
            if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
                // Enum field stored as string - encode as string instead
                let wire_type = 2u32; // Length-delimited
                encode_tag(buffer, field_number, wire_type)?;
                let bytes = arr.value(row_idx).as_bytes();
                encode_varint(buffer, bytes.len() as u64)?;
                buffer.extend_from_slice(bytes);
                Ok(())
            } else if let Some(arr) = array.as_any().downcast_ref::<Int32Array>() {
                // Actual SInt32 value - use zigzag encoding
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                use crate::wrapper::protobuf_serialization::encode_sint32;
                encode_sint32(buffer, arr.value(row_idx))?;
                Ok(())
            } else {
                Err(ZerobusError::ConversionError(format!(
                    "Expected Int32Array or StringArray for SInt32 field '{}', got: {:?}",
                    field_desc.name.as_ref().unwrap_or(&"unknown".to_string()),
                    array.data_type()
                )))
            }
        }
        18 => {
            // SInt64 (signed int64 with zigzag encoding)
            // Often used for enum values
            // Handle case where Arrow has StringArray but descriptor says SInt64 (enum stored as string)
            if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
                // Enum field stored as string - encode as string instead
                let wire_type = 2u32; // Length-delimited
                encode_tag(buffer, field_number, wire_type)?;
                let bytes = arr.value(row_idx).as_bytes();
                encode_varint(buffer, bytes.len() as u64)?;
                buffer.extend_from_slice(bytes);
                Ok(())
            } else if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
                // Actual SInt64 value - use zigzag encoding
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                use crate::wrapper::protobuf_serialization::encode_sint64;
                encode_sint64(buffer, arr.value(row_idx))?;
                Ok(())
            } else {
                Err(ZerobusError::ConversionError(format!(
                    "Expected Int64Array or StringArray for SInt64 field '{}', got: {:?}",
                    field_desc.name.as_ref().unwrap_or(&"unknown".to_string()),
                    array.data_type()
                )))
            }
        }
        _ => {
            // Safety check: type 11 (Message) should never reach encode_arrow_value_to_protobuf
            // If it does, it means the routing logic in encode_arrow_field_to_protobuf failed
            if protobuf_type == 11 {
                let field_name = field_desc.name.as_deref().unwrap_or("unknown");
                let is_repeated_for_log = field_desc.label == Some(Label::Repeated as i32);
                return Err(ZerobusError::ConversionError(format!(
                    "Protobuf type 11 (Message) reached encode_arrow_value_to_protobuf for field '{}'. \
                     This indicates a bug in the routing logic - nested messages should be handled by \
                     encode_arrow_field_to_protobuf. Field descriptor: type={:?}, type_name={:?}, \
                     is_repeated={:?}, label={:?}, array_type={:?}. \
                     Please check the routing logic in encode_arrow_field_to_protobuf.",
                    field_name,
                    protobuf_type,
                    field_desc.type_name,
                    is_repeated_for_log,
                    field_desc.label,
                    array.data_type()
                )));
            }
            Err(ZerobusError::ConversionError(format!(
                "Unsupported Protobuf type: {}",
                protobuf_type
            )))
        }
    }
}

/// Generate Protobuf descriptor from Arrow schema
///
/// Creates a Protobuf DescriptorProto from an Arrow schema.
///
/// # Arguments
///
/// * `schema` - Arrow schema
///
/// # Returns
///
/// Returns DescriptorProto for the schema, or error if generation fails.
pub fn generate_protobuf_descriptor(
    schema: &arrow::datatypes::Schema,
) -> Result<DescriptorProto, ZerobusError> {
    generate_protobuf_descriptor_internal(schema, "ZerobusMessage")
}

/// Internal function to generate Protobuf descriptor with a given message name
fn generate_protobuf_descriptor_internal(
    schema: &arrow::datatypes::Schema,
    message_name: &str,
) -> Result<DescriptorProto, ZerobusError> {
    use prost_types::FieldDescriptorProto;

    let mut fields = Vec::new();
    let mut nested_types = Vec::new();
    let mut field_number = 1;

    for field in schema.fields().iter() {
        // Determine if this is a repeated field (List or LargeList)
        let is_repeated = matches!(
            field.data_type(),
            DataType::List(_) | DataType::LargeList(_)
        );

        // Extract the inner type for lists to determine the actual field type
        let (_inner_data_type, field_type) = match field.data_type() {
            DataType::List(inner_field) | DataType::LargeList(inner_field) => (
                inner_field.data_type(),
                arrow_type_to_protobuf_type(inner_field.data_type())?,
            ),
            _ => (
                field.data_type(),
                arrow_type_to_protobuf_type(field.data_type())?,
            ),
        };

        // Handle nested Struct types (both direct Struct and List<Struct>)
        let type_name = if field_type == Type::Message {
            // Generate nested type descriptor for Struct fields
            // This handles both:
            // 1. Direct Struct fields: DataType::Struct(...)
            // 2. Repeated Struct fields: DataType::List(StructField) or DataType::LargeList(StructField)
            let struct_fields = match field.data_type() {
                DataType::Struct(sf) => sf,
                DataType::List(inner_field) | DataType::LargeList(inner_field) => {
                    // For List<Struct>, extract the Struct fields from the inner type
                    if let DataType::Struct(sf) = inner_field.data_type() {
                        sf
                    } else {
                        return Err(ZerobusError::ConversionError(format!(
                            "List field '{}' contains non-Struct type: {:?}",
                            field.name(),
                            inner_field.data_type()
                        )));
                    }
                }
                _ => {
                    return Err(ZerobusError::ConversionError(format!(
                        "Field '{}' has Message type but is not a Struct or List<Struct>: {:?}",
                        field.name(),
                        field.data_type()
                    )));
                }
            };

            let nested_message_name = format!("{}_{}", message_name, field.name());
            let nested_type_name = format!(".{}.{}", message_name, nested_message_name);

            // Recursively generate descriptor for nested struct
            let nested_schema = arrow::datatypes::Schema::new(struct_fields.clone());
            let nested_descriptor =
                generate_protobuf_descriptor_internal(&nested_schema, &nested_message_name)?;

            nested_types.push(nested_descriptor);
            Some(nested_type_name)
        } else {
            None
        };

        fields.push(FieldDescriptorProto {
            name: Some(field.name().clone()),
            number: Some(field_number),
            label: Some(if is_repeated {
                Label::Repeated as i32
            } else {
                Label::Optional as i32
            }),
            r#type: Some(field_type as i32),
            type_name,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        });

        field_number += 1;
    }

    Ok(DescriptorProto {
        name: Some(message_name.to_string()),
        field: fields,
        extension: vec![],
        nested_type: nested_types,
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    })
}

/// Convert Arrow data type to Protobuf field type
fn arrow_type_to_protobuf_type(
    arrow_type: &arrow::datatypes::DataType,
) -> Result<Type, ZerobusError> {
    use arrow::datatypes::DataType;

    match arrow_type {
        DataType::Int8 | DataType::Int16 | DataType::Int32 => Ok(Type::Int32),
        DataType::Int64 => Ok(Type::Int64),
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 => Ok(Type::Int32), // Protobuf doesn't have unsigned, use Int32
        DataType::UInt64 => Ok(Type::Int64), // Protobuf doesn't have unsigned, use Int64
        DataType::Float32 => Ok(Type::Float),
        DataType::Float64 => Ok(Type::Double),
        DataType::Boolean => Ok(Type::Bool),
        DataType::Utf8 | DataType::LargeUtf8 => Ok(Type::String),
        DataType::Binary | DataType::LargeBinary => Ok(Type::Bytes),
        DataType::Timestamp(_, _) => Ok(Type::Int64), // Store as Int64 (nanoseconds)
        DataType::Date32 | DataType::Date64 => Ok(Type::Int64), // Store as Int64
        DataType::List(inner_type) | DataType::LargeList(inner_type) => {
            // For lists, we need to extract the inner type and convert it
            // Lists in Protobuf are represented as repeated fields
            // The field type will be set to the inner type, and label will be Repeated
            // Note: This is recursive and could theoretically cause infinite recursion
            // if a list contains itself (e.g., List<List>), but this is not a common
            // pattern in Arrow schemas. If needed, a depth check could be added.
            arrow_type_to_protobuf_type(inner_type.data_type())
        }
        DataType::Struct(_) => Ok(Type::Message), // Nested message
        _ => Err(ZerobusError::ConversionError(format!(
            "Unsupported Arrow type: {:?}",
            arrow_type
        ))),
    }
}

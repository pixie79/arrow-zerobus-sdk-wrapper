//! Arrow to Protobuf conversion
//!
//! This module handles conversion of Arrow RecordBatch data to Protobuf format
//! required by Zerobus. Reuses conversion logic from cap-gl-consumer-rust.

use crate::error::ZerobusError;
use crate::wrapper::protobuf_serialization::{encode_tag, encode_varint};
use arrow::array::*;
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use prost_types::{DescriptorProto, FieldDescriptorProto, field_descriptor_proto::Label, field_descriptor_proto::Type};
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
                    None, // descriptor_pool - TODO: support nested types
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
/// * `descriptor_pool` - Optional descriptor pool for nested types
fn encode_arrow_field_to_protobuf(
    buffer: &mut Vec<u8>,
    field_number: i32,
    field_desc: &FieldDescriptorProto,
    array: &Arc<dyn Array>,
    row_idx: usize,
    _parent_descriptor: &DescriptorProto,
    _descriptor_pool: Option<&std::collections::HashMap<String, DescriptorProto>>,
) -> Result<(), ZerobusError> {
    if array.is_null(row_idx) {
        // Protobuf doesn't encode null/optional fields - just skip
        return Ok(());
    }

    let protobuf_type = field_desc.r#type.unwrap_or(9); // Default to String
    let is_repeated = field_desc.label == Some(Label::Repeated as i32);

    // Handle repeated fields
    if is_repeated {
        if let Some(list_array) = array.as_any().downcast_ref::<ListArray>() {
            let offsets = list_array.value_offsets();
            let start = offsets[row_idx] as usize;
            let end = offsets[row_idx + 1] as usize;
            let values = list_array.values();

            // Encode each element in the list
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
    }

    // Handle single nested messages (type 11 = Message)
    if protobuf_type == 11 {
        // TODO: Implement nested message encoding
        // For now, skip nested messages
        debug!("Skipping nested message field (not yet implemented)");
        return Ok(());
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
        1 => { // Double (Float64)
            let arr = array.as_any().downcast_ref::<Float64Array>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected Float64Array".to_string()))?;
            let wire_type = 1u32; // Fixed64
            encode_tag(buffer, field_number, wire_type)?;
            buffer.extend_from_slice(&arr.value(row_idx).to_le_bytes());
            Ok(())
        }
        2 => { // Float (Float32)
            let arr = array.as_any().downcast_ref::<Float32Array>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected Float32Array".to_string()))?;
            let wire_type = 5u32; // Fixed32
            encode_tag(buffer, field_number, wire_type)?;
            buffer.extend_from_slice(&arr.value(row_idx).to_le_bytes());
            Ok(())
        }
        3 => { // Int64
            if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
                let wire_type = 0u32; // Varint
                encode_tag(buffer, field_number, wire_type)?;
                encode_varint(buffer, arr.value(row_idx) as u64)?;
                Ok(())
            } else {
                Err(ZerobusError::ConversionError("Expected Int64Array for Int64 field".to_string()))
            }
        }
        4 => { // UInt64
            let arr = array.as_any().downcast_ref::<UInt64Array>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected UInt64Array".to_string()))?;
            let wire_type = 0u32; // Varint
            encode_tag(buffer, field_number, wire_type)?;
            encode_varint(buffer, arr.value(row_idx))?;
            Ok(())
        }
        5 => { // Int32
            let arr = array.as_any().downcast_ref::<Int32Array>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected Int32Array".to_string()))?;
            let wire_type = 0u32; // Varint
            encode_tag(buffer, field_number, wire_type)?;
            encode_varint(buffer, arr.value(row_idx) as u64)?;
            Ok(())
        }
        8 => { // Bool
            let arr = array.as_any().downcast_ref::<BooleanArray>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected BooleanArray".to_string()))?;
            let wire_type = 0u32; // Varint
            encode_tag(buffer, field_number, wire_type)?;
            encode_varint(buffer, if arr.value(row_idx) { 1 } else { 0 })?;
            Ok(())
        }
        9 => { // String
            let arr = array.as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected StringArray".to_string()))?;
            let wire_type = 2u32; // Length-delimited
            encode_tag(buffer, field_number, wire_type)?;
            let bytes = arr.value(row_idx).as_bytes();
            encode_varint(buffer, bytes.len() as u64)?;
            buffer.extend_from_slice(bytes);
            Ok(())
        }
        12 => { // Bytes
            let arr = array.as_any().downcast_ref::<BinaryArray>()
                .ok_or_else(|| ZerobusError::ConversionError("Expected BinaryArray".to_string()))?;
            let wire_type = 2u32; // Length-delimited
            encode_tag(buffer, field_number, wire_type)?;
            let bytes = arr.value(row_idx);
            encode_varint(buffer, bytes.len() as u64)?;
            buffer.extend_from_slice(bytes);
            Ok(())
        }
        _ => {
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
    use prost_types::FieldDescriptorProto;

    let mut fields = Vec::new();
    let mut field_number = 1;

    for field in schema.fields().iter() {
        let field_type = arrow_type_to_protobuf_type(field.data_type())?;
        let is_repeated = matches!(
            field.data_type(),
            DataType::List(_) | DataType::LargeList(_)
        );

        fields.push(FieldDescriptorProto {
            name: Some(field.name().clone()),
            number: Some(field_number),
            label: Some(if is_repeated {
                Label::Repeated as i32
            } else {
                Label::Optional as i32
            }),
            r#type: Some(field_type as i32),
            type_name: None, // TODO: Handle nested types
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
        name: Some("ZerobusMessage".to_string()),
        field: fields,
        extension: vec![],
        nested_type: vec![],
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
        DataType::List(_) | DataType::LargeList(_) => {
            // For lists, we need to extract the inner type
            // This is a simplified version - full implementation should handle nested types
            Ok(Type::String) // Placeholder
        }
        DataType::Struct(_) => Ok(Type::Message), // Nested message
        _ => Err(ZerobusError::ConversionError(format!(
            "Unsupported Arrow type: {:?}",
            arrow_type
        ))),
    }
}

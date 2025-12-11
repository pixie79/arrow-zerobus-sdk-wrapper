# Rust API Contract: Per-Row Error Information

**Feature**: 004-per-row-errors  
**Date**: 2025-01-27  
**Version**: 0.5.0 (extends 001-zerobus-wrapper)

## Overview

This contract extends the existing Rust API (see `001-zerobus-wrapper/contracts/rust-api.md`) with per-row error tracking capabilities. All existing APIs remain unchanged for backward compatibility.

## Updated API

### TransmissionResult

**Changes**: Extended with optional per-row error tracking fields.

```rust
/// Result of a data transmission operation
#[derive(Debug, Clone)]
pub struct TransmissionResult {
    /// Whether transmission succeeded (true if ANY rows succeeded)
    pub success: bool,
    
    /// Batch-level error if entire batch failed (e.g., authentication, connection failure before row processing)
    pub error: Option<ZerobusError>,
    
    /// Number of retry attempts made
    pub attempts: u32,
    
    /// Transmission latency in milliseconds (if operation completed)
    pub latency_ms: Option<u64>,
    
    /// Size of transmitted batch in bytes
    pub batch_size_bytes: usize,
    
    // NEW: Per-row error tracking fields
    
    /// Indices of rows that failed, along with their specific errors
    /// None if all rows succeeded, Some(vec![]) if no per-row errors (batch-level failure only)
    pub failed_rows: Option<Vec<(usize, ZerobusError)>>,
    
    /// Indices of rows that were successfully written
    /// None if all rows failed, Some(vec![]) if no rows succeeded
    pub successful_rows: Option<Vec<usize>>,
    
    /// Total number of rows in the batch
    pub total_rows: usize,
    
    /// Number of rows that succeeded
    pub successful_count: usize,
    
    /// Number of rows that failed
    pub failed_count: usize,
}
```

**Field Semantics**:

- `success`: `true` if ANY rows succeeded, `false` if ALL rows failed or batch-level error occurred
- `error`: Populated only for batch-level failures (authentication, connection before processing)
- `failed_rows`: `None` if all rows succeeded, `Some(vec![])` if batch-level error only, `Some(vec![...])` for per-row failures
- `successful_rows`: `None` if all rows failed, `Some(vec![])` if no rows succeeded, `Some(vec![...])` for successful rows
- `total_rows`: Always populated with batch size
- `successful_count`: Always populated (0 if all failed)
- `failed_count`: Always populated (0 if all succeeded)

**Consistency Guarantees**:

- `total_rows == successful_count + failed_count`
- If `successful_rows` is `Some`, then `successful_rows.len() == successful_count`
- If `failed_rows` is `Some`, then `failed_rows.len() == failed_count`
- Row indices in `successful_rows` and `failed_rows` are unique and within `[0, total_rows)`

**Helper Methods**:

The `TransmissionResult` struct provides several helper methods for common workflows:

- `is_partial_success()` -> `bool`: Returns `true` if there are both successful and failed rows
- `has_failed_rows()` -> `bool`: Returns `true` if `failed_rows` contains any entries
- `has_successful_rows()` -> `bool`: Returns `true` if `successful_rows` contains any entries
- `get_failed_row_indices()` -> `Vec<usize>`: Returns indices of failed rows
- `get_successful_row_indices()` -> `Vec<usize>`: Returns indices of successful rows
- `extract_failed_batch(batch: &RecordBatch)` -> `Option<RecordBatch>`: Extracts failed rows as a new RecordBatch
- `extract_successful_batch(batch: &RecordBatch)` -> `Option<RecordBatch>`: Extracts successful rows as a new RecordBatch
- `get_failed_row_indices_by_error_type(predicate: F)` -> `Vec<usize>`: Filters failed rows by error type
- `group_errors_by_type()` -> `HashMap<String, Vec<usize>>`: Groups failed rows by error type
- `get_error_statistics()` -> `ErrorStatistics`: Returns comprehensive error statistics
- `get_error_messages()` -> `Vec<String>`: Returns all error messages from failed rows

### ZerobusWrapper

**Changes**: No changes to method signatures. Methods now return `TransmissionResult` with per-row error information populated when applicable.

```rust
impl ZerobusWrapper {
    /// Send a data batch to Zerobus
    /// 
    /// # Arguments
    /// * `batch` - Arrow RecordBatch to send
    /// 
    /// # Returns
    /// TransmissionResult with per-row error information if applicable
    /// 
    /// # Errors
    /// Returns error only if wrapper-level failure occurs (should not happen in normal operation)
    pub async fn send_batch(&self, batch: RecordBatch) -> Result<TransmissionResult, ZerobusError>;
    
    /// Send a data batch to Zerobus with an optional Protobuf descriptor
    /// 
    /// # Arguments
    /// * `batch` - Arrow RecordBatch to send
    /// * `descriptor` - Optional Protobuf descriptor for nested types
    /// 
    /// # Returns
    /// TransmissionResult with per-row error information if applicable
    /// 
    /// # Errors
    /// Returns error only if wrapper-level failure occurs (should not happen in normal operation)
    pub async fn send_batch_with_descriptor(
        &self,
        batch: RecordBatch,
        descriptor: Option<prost_types::DescriptorProto>,
    ) -> Result<TransmissionResult, ZerobusError>;
    
    // ... other methods unchanged
}
```

### Internal API Changes

**Note**: These are internal implementation details, documented for completeness.

#### ProtobufConversionResult

**Status**: Already exists in `src/wrapper/conversion.rs`, will be used by modified conversion function.

```rust
/// Result of converting a RecordBatch to Protobuf
#[derive(Debug)]
pub struct ProtobufConversionResult {
    /// Successful conversions: (row_index, protobuf_bytes)
    pub successful_bytes: Vec<(usize, Vec<u8>)>,
    /// Failed conversions: (row_index, error_message)
    pub failed_rows: Vec<(usize, String)>,
}
```

**Change**: `record_batch_to_protobuf_bytes` will be modified to return `ProtobufConversionResult` instead of `Result<Vec<Vec<u8>>, ZerobusError>`.

## Usage Examples

### Basic Usage (Backward Compatible)

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use arrow::record_batch::RecordBatch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WrapperConfiguration::new(
        "https://workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    );
    
    let wrapper = ZerobusWrapper::new(config).await?;
    let batch = create_record_batch()?;
    
    // Existing code continues to work
    let result = wrapper.send_batch(batch).await?;
    
    if result.success {
        println!("Batch sent successfully");
    } else {
        eprintln!("Transmission failed: {:?}", result.error);
    }
    
    Ok(())
}
```

### Per-Row Error Handling

```rust
let result = wrapper.send_batch(batch).await?;

if result.success {
    // Some or all rows succeeded
    if let Some(successful_rows) = &result.successful_rows {
        println!("Successfully wrote {} rows", successful_rows.len());
        // Process successful rows
        for &row_idx in successful_rows {
            println!("Row {} succeeded", row_idx);
        }
    }
    
    // Handle failed rows separately
    if let Some(failed_rows) = &result.failed_rows {
        for (row_idx, error) in failed_rows {
            eprintln!("Row {} failed: {:?}", row_idx, error);
            // Quarantine only this row
            quarantine_row(&batch, *row_idx, error).await?;
        }
    }
} else {
    // Entire batch failed (batch-level error)
    eprintln!("Batch failed: {:?}", result.error);
    // Quarantine all rows
    quarantine_batch(&batch).await?;
}

// Use counts for quick checks
println!("Total: {}, Success: {}, Failed: {}", 
    result.total_rows, result.successful_count, result.failed_count);
```

### Partial Success Handling

```rust
let result = wrapper.send_batch(batch).await?;

match (result.success, result.failed_rows.as_ref()) {
    (true, None) => {
        // All rows succeeded
        println!("All {} rows succeeded", result.total_rows);
    },
    (true, Some(failed)) if !failed.is_empty() => {
        // Partial success
        println!("Partial success: {} succeeded, {} failed", 
            result.successful_count, result.failed_count);
        
        // Write successful rows to main table
        if let Some(successful) = &result.successful_rows {
            write_rows_to_table(&batch, successful).await?;
        }
        
        // Quarantine failed rows
        for (row_idx, error) in failed {
            quarantine_row(&batch, *row_idx, error).await?;
        }
    },
    (false, _) => {
        // All rows failed or batch-level error
        if let Some(error) = &result.error {
            eprintln!("Batch-level error: {:?}", error);
        }
        quarantine_batch(&batch).await?;
    },
}
```

## Error Types in Per-Row Context

Per-row errors use the same `ZerobusError` enum variants with row-specific context:

- `ConversionError(String)`: Row failed during Arrow-to-Protobuf conversion
  - Example: `"Field encoding failed: field='name', row=5, error=type mismatch"`
  
- `TransmissionError(String)`: Row failed during transmission
  - Example: `"Record ingestion failed: row=10, error=timeout"`
  
- `ConnectionError(String)`: Row failed due to connection issue
  - Example: `"Stream closed: row=3, error=connection reset"`

Batch-level errors (authentication, connection before processing) continue to use the `error` field.

## Backward Compatibility

✅ **Fully backward compatible**: All existing code continues to work without modification.

- Existing code that checks `result.success` continues to work
- Existing code that accesses `result.error` continues to work
- New fields are `Option<>` types, so they can be ignored
- Default values ensure existing code doesn't break

## Performance Characteristics

- **Overhead for successful batches**: < 1% (minimal checking, no error collection)
- **Overhead for batches with failures**: < 10% (error collection and tracking)
- **Memory overhead**: O(failed_rows + successful_rows) - bounded by batch size
- **Latency impact**: Negligible for typical batch sizes (100-20,000 rows)

## Thread Safety

All methods remain thread-safe. Per-row error tracking uses internal data structures that are safely shared across concurrent operations.

## Versioning

This is a backward-compatible extension to the existing API. No version bump required for existing functionality. New fields are additive and optional.

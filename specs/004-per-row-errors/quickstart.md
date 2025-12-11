# Quickstart: Per-Row Error Information

**Feature**: 004-per-row-errors  
**Date**: 2025-01-27

## Overview

This feature adds per-row error tracking to `TransmissionResult`, enabling you to identify which specific rows failed during batch transmission. This allows efficient quarantine workflows that only quarantine failed rows while successfully writing valid rows to the main table.

## Key Benefits

1. **Prevent Data Loss**: Only quarantine rows that actually failed, not entire batches
2. **Partial Success**: Write successful rows even if some rows fail
3. **Better Debugging**: Identify which rows failed and why
4. **Efficient Processing**: Reduce unnecessary data loss in production systems

## Quick Example (Rust)

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use arrow::record_batch::RecordBatch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize wrapper
    let config = WrapperConfiguration::new(
        "https://workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    );
    let wrapper = ZerobusWrapper::new(config).await?;
    
    // Send batch
    let batch = create_record_batch()?;
    let result = wrapper.send_batch(batch).await?;
    
    // Check for partial success
    if result.success {
        // Some or all rows succeeded
        if let Some(successful_rows) = &result.successful_rows {
            println!("✅ {} rows succeeded", successful_rows.len());
        }
        
        // Handle failed rows
        if let Some(failed_rows) = &result.failed_rows {
            println!("❌ {} rows failed", failed_rows.len());
            for (row_idx, error) in failed_rows {
                println!("  Row {}: {:?}", row_idx, error);
                // Quarantine only this row
                quarantine_row(&batch, *row_idx, error).await?;
            }
        }
    } else {
        // Entire batch failed
        println!("Batch failed: {:?}", result.error);
    }
    
    Ok(())
}
```

## Quick Example (Python)

```python
import asyncio
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def main():
    # Initialize wrapper
    wrapper = ZerobusWrapper(
        endpoint="https://workspace.cloud.databricks.com",
        table_name="my_table",
        client_id="client_id",
        client_secret="client_secret",
    )
    
    # Send batch
    batch = create_record_batch()
    result = await wrapper.send_batch(batch)
    
    # Check for partial success
    if result.success:
        # Some or all rows succeeded
        if result.successful_rows:
            print(f"✅ {len(result.successful_rows)} rows succeeded")
            # Write successful rows to main table
            await write_rows_to_table(batch, result.successful_rows)
        
        # Handle failed rows
        if result.failed_rows:
            print(f"❌ {len(result.failed_rows)} rows failed")
            for row_idx, error_msg in result.failed_rows:
                print(f"  Row {row_idx}: {error_msg}")
                # Quarantine only this row
                await quarantine_row(batch, row_idx, error_msg)
    else:
        # Entire batch failed
        print(f"Batch failed: {result.error}")
    
    await wrapper.shutdown()

asyncio.run(main())
```

## Understanding TransmissionResult

### New Fields

The `TransmissionResult` struct now includes these optional fields:

| Field | Type | Description |
|-------|------|-------------|
| `failed_rows` | `Option<Vec<(usize, ZerobusError)>>` | Failed rows with errors |
| `successful_rows` | `Option<Vec<usize>>` | Successful row indices |
| `total_rows` | `usize` | Total rows in batch |
| `successful_count` | `usize` | Count of successful rows |
| `failed_count` | `usize` | Count of failed rows |

### Field Semantics

- **`success`**: `true` if ANY rows succeeded, `false` if ALL rows failed
- **`error`**: Batch-level error (authentication, connection before processing)
- **`failed_rows`**: `None` if all succeeded, `Some(vec![])` if batch-level only, `Some(vec![...])` for per-row failures
- **`successful_rows`**: `None` if all failed, `Some(vec![])` if none succeeded, `Some(vec![...])` for successful rows

## Common Patterns

### Pattern 1: Quarantine Failed Rows Only (Using Helper Methods)

```rust
let result = wrapper.send_batch(batch).await?;

if result.success {
    // Extract and write successful rows to main table
    if let Some(successful_batch) = result.extract_successful_batch(&batch) {
        write_to_main_table(successful_batch).await?;
    }
    
    // Extract and quarantine failed rows
    if let Some(failed_batch) = result.extract_failed_batch(&batch) {
        quarantine_batch(failed_batch).await?;
        
        // Optionally, log specific errors
        for (row_idx, error) in result.failed_rows.as_ref().unwrap() {
            eprintln!("Row {} failed: {:?}", row_idx, error);
        }
    }
}
```

**Python equivalent:**
```python
result = await wrapper.send_batch(batch)

if result.success:
    # Extract and write successful rows to main table
    successful_batch = result.extract_successful_batch(batch)
    if successful_batch is not None:
        await write_to_main_table(successful_batch)
    
    # Extract and quarantine failed rows
    failed_batch = result.extract_failed_batch(batch)
    if failed_batch is not None:
        await quarantine_batch(failed_batch)
        
        # Optionally, log specific errors
        if result.failed_rows:
            for row_idx, error_msg in result.failed_rows:
                print(f"Row {row_idx} failed: {error_msg}")
```

### Pattern 2: Error Analysis and Statistics (Using Helper Methods)

```rust
let result = wrapper.send_batch(batch).await?;

// Get comprehensive error statistics
let stats = result.get_error_statistics();
println!("Batch Statistics:");
println!("  Total rows: {}", stats.total_rows);
println!("  Successful: {} ({:.1}%)", stats.successful_count, stats.success_rate * 100.0);
println!("  Failed: {} ({:.1}%)", stats.failed_count, stats.failure_rate * 100.0);

// Group errors by type using helper method
let grouped = result.group_errors_by_type();
if !grouped.is_empty() {
    println!("Error breakdown by type:");
    for (error_type, indices) in &grouped {
        println!("  {}: {} rows (indices: {:?})", error_type, indices.len(), indices);
    }
}

// Get all error messages for debugging
let error_messages = result.get_error_messages();
if !error_messages.is_empty() {
    println!("Error messages:");
    for (i, msg) in error_messages.iter().enumerate() {
        println!("  {}. {}", i + 1, msg);
    }
}
```

**Python equivalent:**
```python
result = await wrapper.send_batch(batch)

# Get comprehensive error statistics
stats = result.get_error_statistics()
print("Batch Statistics:")
print(f"  Total rows: {stats['total_rows']}")
print(f"  Successful: {stats['successful_count']} ({stats['success_rate'] * 100:.1}%)")
print(f"  Failed: {stats['failed_count']} ({stats['failure_rate'] * 100:.1}%)")

# Group errors by type using helper method
grouped = result.group_errors_by_type()
if grouped:
    print("Error breakdown by type:")
    for error_type, indices in grouped.items():
        print(f"  {error_type}: {len(indices)} rows (indices: {indices})")

# Get all error messages for debugging
error_messages = result.get_error_messages()
if error_messages:
    print("Error messages:")
    for i, msg in enumerate(error_messages, 1):
        print(f"  {i}. {msg}")
```

### Pattern 3: Retry Failed Rows (Using Helper Methods)

```rust
let result = wrapper.send_batch(batch).await?;

if result.has_failed_rows() {
    // Extract failed rows using helper method
    if let Some(failed_batch) = result.extract_failed_batch(&batch) {
        // Retry failed rows
        let retry_result = wrapper.send_batch(failed_batch).await?;
        
        // Handle retry results
        if retry_result.success {
            println!("Retry succeeded for {} rows", retry_result.successful_count);
        }
    }
}
```

**Python equivalent:**
```python
result = await wrapper.send_batch(batch)

if result.has_failed_rows():
    # Extract failed rows using helper method
    failed_batch = result.extract_failed_batch(batch)
    if failed_batch is not None:
        # Retry failed rows
        retry_result = await wrapper.send_batch(failed_batch)
        
        # Handle retry results
        if retry_result.success:
            print(f"Retry succeeded for {retry_result.successful_count} rows")
```

### Pattern 4: Filter Failed Rows by Error Type

```rust
let result = wrapper.send_batch(batch).await?;

if result.has_failed_rows() {
    // Get only conversion errors
    let conversion_error_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::ConversionError(_))
    });
    
    // Get only transmission errors
    let transmission_error_indices = result.get_failed_row_indices_by_error_type(|e| {
        matches!(e, ZerobusError::TransmissionError(_))
    });
    
    println!("Conversion errors: {} rows", conversion_error_indices.len());
    println!("Transmission errors: {} rows", transmission_error_indices.len());
}
```

**Python equivalent:**
```python
result = await wrapper.send_batch(batch)

if result.has_failed_rows():
    # Get only conversion errors
    conversion_error_indices = result.get_failed_row_indices_by_error_type("ConversionError")
    
    # Get only transmission errors
    transmission_error_indices = result.get_failed_row_indices_by_error_type("TransmissionError")
    
    print(f"Conversion errors: {len(conversion_error_indices)} rows")
    print(f"Transmission errors: {len(transmission_error_indices)} rows")
```

### Pattern 5: Monitor Error Patterns Over Time

```rust
// Collect results from multiple batches
let mut all_stats = Vec::new();
for batch in batches {
    let result = wrapper.send_batch(batch).await?;
    all_stats.push(result.get_error_statistics());
}

// Aggregate statistics
let total_rows: usize = all_stats.iter().map(|s| s.total_rows).sum();
let total_successful: usize = all_stats.iter().map(|s| s.successful_count).sum();
let total_failed: usize = all_stats.iter().map(|s| s.failed_count).sum();

let overall_success_rate = total_successful as f64 / total_rows as f64;
println!("Overall success rate: {:.1}%", overall_success_rate * 100.0);

// Aggregate error type counts
let mut error_type_totals = std::collections::HashMap::new();
for stats in &all_stats {
    for (error_type, count) in &stats.error_type_counts {
        *error_type_totals.entry(error_type.clone()).or_insert(0) += count;
    }
}

println!("Error type distribution:");
for (error_type, count) in error_type_totals {
    println!("  {}: {} occurrences", error_type, count);
}
```

**Python equivalent:**
```python
# Collect results from multiple batches
all_stats = []
for batch in batches:
    result = await wrapper.send_batch(batch)
    all_stats.append(result.get_error_statistics())

# Aggregate statistics
total_rows = sum(s["total_rows"] for s in all_stats)
total_successful = sum(s["successful_count"] for s in all_stats)
total_failed = sum(s["failed_count"] for s in all_stats)

overall_success_rate = total_successful / total_rows if total_rows > 0 else 0.0
print(f"Overall success rate: {overall_success_rate * 100:.1}%")

# Aggregate error type counts
error_type_totals = {}
for stats in all_stats:
    for error_type, count in stats["error_type_counts"].items():
        error_type_totals[error_type] = error_type_totals.get(error_type, 0) + count

print("Error type distribution:")
for error_type, count in error_type_totals.items():
    print(f"  {error_type}: {count} occurrences")
```

## Backward Compatibility

✅ **Fully backward compatible**: All existing code continues to work without modification.

```rust
// Existing code still works
let result = wrapper.send_batch(batch).await?;
if result.success {
    println!("Success!");
} else {
    println!("Failed: {:?}", result.error);
}
```

New fields are optional and can be ignored if not needed.

## Performance Considerations

- **Successful batches**: < 1% overhead (minimal checking)
- **Batches with failures**: < 10% overhead (error collection)
- **Memory**: Bounded by batch size (O(failed_rows + successful_rows))

## Error Types

Per-row errors use the same `ZerobusError` enum with row-specific context:

- **`ConversionError`**: Row failed during Arrow-to-Protobuf conversion
  - Example: `"Field encoding failed: field='name', row=5, error=type mismatch"`
  
- **`TransmissionError`**: Row failed during transmission
  - Example: `"Record ingestion failed: row=10, error=timeout"`
  
- **`ConnectionError`**: Row failed due to connection issue
  - Example: `"Stream closed: row=3, error=connection reset"`

## Next Steps

1. **Read the full API documentation**: See `contracts/rust-api.md` or `contracts/python-api.md`
2. **Review the data model**: See `data-model.md` for detailed field semantics
3. **Check examples**: See `examples/` directory for complete examples
4. **Implement quarantine logic**: Use per-row errors to quarantine only failed rows

## Troubleshooting

### Q: Why are `failed_rows` and `successful_rows` `None`?

A: They are `None` when:
- `failed_rows` is `None` if all rows succeeded
- `successful_rows` is `None` if all rows failed

Check `successful_count` and `failed_count` for quick status.

### Q: What's the difference between `error` and `failed_rows`?

A: 
- `error`: Batch-level error (affects entire batch, prevents row processing)
- `failed_rows`: Per-row errors (individual rows failed, partial success possible)

### Q: Can I use this with existing code?

A: Yes! All new fields are optional. Existing code that checks `result.success` continues to work.

## Related Documentation

- **Specification**: `spec.md` - Full feature specification
- **Research**: `research.md` - Technical decisions and rationale
- **Data Model**: `data-model.md` - Detailed data structure definitions
- **Rust API**: `contracts/rust-api.md` - Complete Rust API documentation
- **Python API**: `contracts/python-api.md` - Complete Python API documentation

# Python API Contract: Per-Row Error Information

**Feature**: 004-per-row-errors  
**Date**: 2025-01-27  
**Version**: 0.5.0 (extends 001-zerobus-wrapper)

## Overview

This contract extends the existing Python API (see `001-zerobus-wrapper/contracts/python-api.md`) with per-row error tracking capabilities. All existing APIs remain unchanged for backward compatibility.

## Updated API

### TransmissionResult

**Changes**: Extended with optional per-row error tracking fields.

```python
@dataclass
class TransmissionResult:
    """Result of a data transmission operation."""
    
    success: bool
    """Whether transmission succeeded (True if ANY rows succeeded)."""
    
    error: Optional[str]
    """Batch-level error message if entire batch failed (e.g., authentication, connection failure before row processing)."""
    
    attempts: int
    """Number of retry attempts made."""
    
    latency_ms: Optional[int]
    """Transmission latency in milliseconds (if operation completed)."""
    
    batch_size_bytes: int
    """Size of transmitted batch in bytes."""
    
    # NEW: Per-row error tracking fields
    
    failed_rows: Optional[List[Tuple[int, str]]]
    """Indices of rows that failed, along with their specific error messages.
    
    None if all rows succeeded, empty list if no per-row errors (batch-level failure only).
    Each tuple is (row_index, error_message).
    """
    
    successful_rows: Optional[List[int]]
    """Indices of rows that were successfully written.
    
    None if all rows failed, empty list if no rows succeeded.
    """
    
    total_rows: int
    """Total number of rows in the batch."""
    
    successful_count: int
    """Number of rows that succeeded."""
    
    failed_count: int
    """Number of rows that failed."""
```

**Field Semantics**:

- `success`: `True` if ANY rows succeeded, `False` if ALL rows failed or batch-level error occurred
- `error`: Populated only for batch-level failures (authentication, connection before processing)
- `failed_rows`: `None` if all rows succeeded, `[]` if batch-level error only, `[(idx, msg), ...]` for per-row failures
- `successful_rows`: `None` if all rows failed, `[]` if no rows succeeded, `[idx, ...]` for successful rows
- `total_rows`: Always populated with batch size
- `successful_count`: Always populated (0 if all failed)
- `failed_count`: Always populated (0 if all succeeded)

**Consistency Guarantees**:

- `total_rows == successful_count + failed_count`
- If `successful_rows` is not `None`, then `len(successful_rows) == successful_count`
- If `failed_rows` is not `None`, then `len(failed_rows) == failed_count`
- Row indices in `successful_rows` and `failed_rows` are unique and within `[0, total_rows)`

**Helper Methods**:

The `TransmissionResult` class provides several helper methods for common workflows:

- `is_partial_success()` -> `bool`: Returns `True` if there are both successful and failed rows
- `has_failed_rows()` -> `bool`: Returns `True` if `failed_rows` contains any entries
- `has_successful_rows()` -> `bool`: Returns `True` if `successful_rows` contains any entries
- `get_failed_row_indices()` -> `List[int]`: Returns indices of failed rows
- `get_successful_row_indices()` -> `List[int]`: Returns indices of successful rows
- `extract_failed_batch(batch: pyarrow.RecordBatch)` -> `Optional[pyarrow.RecordBatch]`: Extracts failed rows as a new RecordBatch
- `extract_successful_batch(batch: pyarrow.RecordBatch)` -> `Optional[pyarrow.RecordBatch]`: Extracts successful rows as a new RecordBatch
- `get_failed_row_indices_by_error_type(error_type: str)` -> `List[int]`: Filters failed rows by error type
- `group_errors_by_type()` -> `Dict[str, List[int]]`: Groups failed rows by error type
- `get_error_statistics()` -> `Dict`: Returns comprehensive error statistics (total_rows, successful_count, failed_count, success_rate, failure_rate, error_type_counts)
- `get_error_messages()` -> `List[str]`: Returns all error messages from failed rows

### ZerobusWrapper

**Changes**: No changes to method signatures. Methods now return `TransmissionResult` with per-row error information populated when applicable.

```python
class ZerobusWrapper:
    """Wrapper for sending Arrow RecordBatch data to Databricks Zerobus.
    
    Thread-safe and supports concurrent operations from multiple threads.
    """
    
    async def send_batch(self, batch: pyarrow.RecordBatch) -> TransmissionResult:
        """Send an Arrow RecordBatch to Zerobus.
        
        Args:
            batch: PyArrow RecordBatch to send
        
        Returns:
            TransmissionResult with per-row error information if applicable
        
        Raises:
            ZerobusError: Only if wrapper-level failure occurs (should not happen in normal operation)
        """
    
    # ... other methods unchanged
```

## Usage Examples

### Basic Usage (Backward Compatible)

```python
import asyncio
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def main():
    wrapper = ZerobusWrapper(
        endpoint="https://workspace.cloud.databricks.com",
        table_name="my_table",
        client_id="client_id",
        client_secret="client_secret",
    )
    
    batch = create_record_batch()
    
    # Existing code continues to work
    result = await wrapper.send_batch(batch)
    
    if result.success:
        print("Batch sent successfully")
    else:
        print(f"Transmission failed: {result.error}")
    
    await wrapper.shutdown()
```

### Per-Row Error Handling

```python
result = await wrapper.send_batch(batch)

if result.success:
    # Some or all rows succeeded
    if result.successful_rows:
        print(f"Successfully wrote {len(result.successful_rows)} rows")
        # Process successful rows
        for row_idx in result.successful_rows:
            print(f"Row {row_idx} succeeded")
    
    # Handle failed rows separately
    if result.failed_rows:
        for row_idx, error_msg in result.failed_rows:
            print(f"Row {row_idx} failed: {error_msg}")
            # Quarantine only this row
            await quarantine_row(batch, row_idx, error_msg)
else:
    # Entire batch failed (batch-level error)
    print(f"Batch failed: {result.error}")
    # Quarantine all rows
    await quarantine_batch(batch)

# Use counts for quick checks
print(f"Total: {result.total_rows}, Success: {result.successful_count}, Failed: {result.failed_count}")
```

### Partial Success Handling

```python
result = await wrapper.send_batch(batch)

if result.success and result.failed_rows:
    # Partial success
    print(f"Partial success: {result.successful_count} succeeded, {result.failed_count} failed")
    
    # Write successful rows to main table
    if result.successful_rows:
        await write_rows_to_table(batch, result.successful_rows)
    
    # Quarantine failed rows
    for row_idx, error_msg in result.failed_rows:
        await quarantine_row(batch, row_idx, error_msg)
        
elif result.success:
    # All rows succeeded
    print(f"All {result.total_rows} rows succeeded")
    
else:
    # All rows failed or batch-level error
    if result.error:
        print(f"Batch-level error: {result.error}")
    await quarantine_batch(batch)
```

### Error Pattern Analysis

```python
result = await wrapper.send_batch(batch)

if result.failed_rows:
    # Analyze error patterns
    error_types = {}
    for row_idx, error_msg in result.failed_rows:
        # Extract error type from message
        if "ConversionError" in error_msg:
            error_types["conversion"] = error_types.get("conversion", 0) + 1
        elif "TransmissionError" in error_msg:
            error_types["transmission"] = error_types.get("transmission", 0) + 1
        elif "ConnectionError" in error_msg:
            error_types["connection"] = error_types.get("connection", 0) + 1
    
    print(f"Error distribution: {error_types}")
```

### Type Hints

```python
from typing import Optional, List, Tuple
from arrow_zerobus_sdk_wrapper import TransmissionResult

def process_result(result: TransmissionResult) -> None:
    """Process transmission result with type hints."""
    
    # Type checker knows these are Optional
    if result.failed_rows is not None:
        # Type checker knows this is List[Tuple[int, str]]
        for row_idx, error_msg in result.failed_rows:
            print(f"Row {row_idx}: {error_msg}")
    
    if result.successful_rows is not None:
        # Type checker knows this is List[int]
        for row_idx in result.successful_rows:
            print(f"Row {row_idx} succeeded")
```

## Error Messages in Per-Row Context

Per-row error messages are strings that include row context:

- **ConversionError**: `"Field encoding failed: field='name', row=5, error=type mismatch"`
- **TransmissionError**: `"Record ingestion failed: row=10, error=timeout"`
- **ConnectionError**: `"Stream closed: row=3, error=connection reset"`

Batch-level errors (authentication, connection before processing) continue to use the `error` field.

## Backward Compatibility

✅ **Fully backward compatible**: All existing code continues to work without modification.

- Existing code that checks `result.success` continues to work
- Existing code that accesses `result.error` continues to work
- New fields are `Optional` types, so they can be ignored
- Default values ensure existing code doesn't break

## Performance Characteristics

- **Overhead for successful batches**: < 1% (minimal checking, no error collection)
- **Overhead for batches with failures**: < 10% (error collection and tracking)
- **Memory overhead**: O(failed_rows + successful_rows) - bounded by batch size
- **Latency impact**: Negligible for typical batch sizes (100-20,000 rows)

## Thread Safety

All methods remain thread-safe. Per-row error tracking uses internal data structures that are safely shared across concurrent operations.

## Dependencies

- Python 3.11+
- PyArrow (for Arrow RecordBatch support)
- asyncio (for async operations)
- typing (for type hints)

## Installation

```bash
pip install arrow-zerobus-sdk-wrapper
```

## Versioning

This is a backward-compatible extension to the existing API. No version bump required for existing functionality. New fields are additive and optional.

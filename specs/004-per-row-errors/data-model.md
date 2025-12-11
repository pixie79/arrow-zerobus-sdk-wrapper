# Data Model: Per-Row Error Information

**Feature**: 004-per-row-errors  
**Date**: 2025-01-27

## Overview

This document defines the data structures used to track per-row errors in batch transmission operations. The model extends the existing `TransmissionResult` structure with optional per-row error tracking fields.

## Core Entities

### TransmissionResult

**Purpose**: Result structure returned from batch transmission operations, indicating success/failure status and providing per-row error details.

**Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `success` | `bool` | Yes | Batch-level success status. `true` if ANY rows succeeded, `false` if ALL rows failed or batch-level error occurred |
| `error` | `Option<ZerobusError>` | Yes | Batch-level error if entire batch failed (e.g., authentication, connection failure before row processing) |
| `attempts` | `u32` | Yes | Number of retry attempts made at batch level |
| `latency_ms` | `Option<u64>` | Yes | Transmission latency in milliseconds (if operation completed) |
| `batch_size_bytes` | `usize` | Yes | Size of transmitted batch in bytes |
| `failed_rows` | `Option<Vec<(usize, ZerobusError)>>` | No | **NEW**: Per-row failures. Vector of tuples: (row_index, error). Empty/None if all rows succeeded |
| `successful_rows` | `Option<Vec<usize>>` | No | **NEW**: Indices of successfully written rows. Empty/None if all rows failed |
| `total_rows` | `usize` | No | **NEW**: Total number of rows in the batch |
| `successful_count` | `usize` | No | **NEW**: Count of successfully written rows |
| `failed_count` | `usize` | No | **NEW**: Count of failed rows |

**Validation Rules**:

1. If `success == true`:
   - `successful_count > 0` (at least one row succeeded)
   - `failed_rows` may be `None` or contain entries (partial success)
   - `error` should be `None` unless there was also a batch-level warning

2. If `success == false`:
   - Either `error` is `Some` (batch-level failure) OR `failed_count == total_rows` (all rows failed)
   - `successful_rows` should be `None` or empty
   - `successful_count == 0`

3. Consistency checks:
   - `total_rows == successful_count + failed_count`
   - If `successful_rows` is `Some`, `successful_rows.len() == successful_count`
   - If `failed_rows` is `Some`, `failed_rows.len() == failed_count`
   - Row indices in `successful_rows` and `failed_rows` should be unique and within range `[0, total_rows)`

**State Transitions**:

- **Initial**: All fields initialized with default/empty values
- **During Processing**: Per-row errors collected as rows are processed
- **Final**: All fields populated based on processing results

**Edge Cases**:

- **Empty batch** (`total_rows == 0`): `successful_count == 0`, `failed_count == 0`, `successful_rows == None`, `failed_rows == None`
- **All rows succeed**: `failed_rows == None` or empty, `successful_count == total_rows`, `failed_count == 0`
- **All rows fail**: `successful_rows == None` or empty, `failed_count == total_rows`, `successful_count == 0`
- **Batch-level error**: `error` is `Some`, `failed_rows` may be `None` (no row processing occurred)

---

### FailedRowEntry

**Purpose**: Represents a single failed row with its index and error information.

**Structure**: `(usize, ZerobusError)` - Tuple of row index and error

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `row_index` | `usize` | 0-based index of the failed row in the original batch |
| `error` | `ZerobusError` | Specific error that occurred for this row |

**Validation Rules**:

1. `row_index` must be within valid range `[0, total_rows)`
2. `error` must be a valid `ZerobusError` variant
3. Row indices in `failed_rows` vector should be unique (no duplicate entries)

**Error Types** (from `ZerobusError` enum):

- `ConversionError`: Row failed during Arrow-to-Protobuf conversion (serialization error, type mismatch, missing descriptor)
- `TransmissionError`: Row failed during transmission to Zerobus (network error, timeout)
- `ConnectionError`: Row failed due to connection issue (stream closed, connection reset)
- Other error types may apply depending on failure scenario

---

### SuccessfulRowEntry

**Purpose**: Represents a successfully written row by its index.

**Structure**: `usize` - Row index

**Validation Rules**:

1. Row index must be within valid range `[0, total_rows)`
2. Row indices in `successful_rows` vector should be unique (no duplicate entries)
3. Row indices should not overlap with `failed_rows` indices

---

### ZerobusError

**Purpose**: Error type representing various failure scenarios. (No changes to existing enum)

**Variants** (existing):

- `ConfigurationError(String)`: Invalid configuration
- `AuthenticationError(String)`: Authentication failure
- `ConnectionError(String)`: Network/connection error
- `ConversionError(String)`: Arrow to Protobuf conversion failure
- `TransmissionError(String)`: Data transmission failure
- `RetryExhausted(String)`: All retry attempts exhausted
- `TokenRefreshError(String)`: Token refresh failure

**Usage in Per-Row Context**:

- `ConversionError`: Used for per-row conversion failures (e.g., "Field encoding failed: field='name', row=5, error=...")
- `TransmissionError`: Used for per-row transmission failures (e.g., "Record ingestion failed: row=10, error=...")
- `ConnectionError`: Used for per-row connection failures (e.g., "Stream closed: row=3, error=...")
- Other error types typically used for batch-level errors

---

## Relationships

```
TransmissionResult
├── error: Option<ZerobusError> (batch-level)
├── failed_rows: Option<Vec<FailedRowEntry>>
│   └── FailedRowEntry: (usize, ZerobusError)
├── successful_rows: Option<Vec<usize>>
└── counts: total_rows, successful_count, failed_count
```

**Relationship Rules**:

1. `TransmissionResult` contains zero or more `FailedRowEntry` instances
2. `TransmissionResult` contains zero or more successful row indices
3. Each `FailedRowEntry` references a specific row index and error
4. Row indices in `successful_rows` and `failed_rows` are mutually exclusive
5. Sum of `successful_count` and `failed_count` equals `total_rows`

---

## Data Flow

### Conversion Phase

```
RecordBatch (N rows)
    ↓
record_batch_to_protobuf_bytes()
    ↓
ProtobufConversionResult
├── successful_bytes: Vec<(usize, Vec<u8>)>
└── failed_rows: Vec<(usize, String)>
```

### Transmission Phase

```
ProtobufConversionResult
    ↓
send_batch_internal()
    ↓
Track per-row transmission results
    ↓
Merge conversion errors + transmission errors
    ↓
TransmissionResult
├── failed_rows: Vec<(usize, ZerobusError)>
└── successful_rows: Vec<usize>
```

---

## Python Bindings Representation

### TransmissionResult (Python)

**Structure**: Python class or dict-like object

**Fields** (Python types):

| Field | Python Type | Description |
|-------|-------------|-------------|
| `success` | `bool` | Batch-level success status |
| `error` | `Optional[str]` or `Optional[ZerobusError]` | Batch-level error message |
| `attempts` | `int` | Number of retry attempts |
| `latency_ms` | `Optional[int]` | Latency in milliseconds |
| `batch_size_bytes` | `int` | Batch size in bytes |
| `failed_rows` | `Optional[List[Tuple[int, str]]]` | List of (row_index, error_message) tuples |
| `successful_rows` | `Optional[List[int]]` | List of successful row indices |
| `total_rows` | `int` | Total rows in batch |
| `successful_count` | `int` | Count of successful rows |
| `failed_count` | `int` | Count of failed rows |

**Example**:

```python
result = wrapper.send_batch(batch)
# result.success -> bool
# result.failed_rows -> [(5, "ConversionError: ..."), (10, "TransmissionError: ...")]
# result.successful_rows -> [0, 1, 2, 3, 4, 6, 7, 8, 9, 11, ...]
```

---

## Validation and Constraints

### Input Validation

- Batch must contain at least 0 rows (empty batch is valid)
- Row indices must be 0-based and sequential in input batch

### Output Validation

- All row indices in `successful_rows` and `failed_rows` must be valid (within `[0, total_rows)`)
- No duplicate row indices across `successful_rows` and `failed_rows`
- Counts must be consistent: `total_rows == successful_count + failed_count`
- Vector lengths must match counts: `successful_rows.len() == successful_count`, `failed_rows.len() == failed_count`

### Performance Constraints

- Memory overhead: O(failed_rows + successful_rows) - bounded by batch size
- Collection overhead: Minimal for successful batches (no error collection)
- Processing overhead: < 10% compared to batch-level error handling

---

## Migration Notes

### Backward Compatibility

- All new fields are optional (`Option<>` in Rust, `Optional` in Python)
- Existing code that doesn't access new fields continues to work
- `success` field behavior unchanged (true if ANY rows succeeded)
- `error` field still used for batch-level errors

### Breaking Changes

None - this is a backward-compatible extension.

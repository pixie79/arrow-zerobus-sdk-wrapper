# Research: Per-Row Error Information in TransmissionResult

**Feature**: 004-per-row-errors  
**Date**: 2025-01-27  
**Purpose**: Document technical decisions and research findings for implementing per-row error tracking

## Research Questions

### Q1: How should we modify the conversion process to collect per-row errors instead of failing fast?

**Decision**: Modify `record_batch_to_protobuf_bytes` to collect errors per row and return a result structure that includes both successful and failed rows.

**Rationale**: 
- Current implementation fails immediately on first conversion error (line 174-179 in `conversion.rs`)
- We need to process all rows and collect errors for each failed row
- There's already a `ProtobufConversionResult` struct (lines 93-99) that has the structure we need, but it's not currently used
- We should modify the conversion function to return `ProtobufConversionResult` instead of `Result<Vec<Vec<u8>>, ZerobusError>`

**Alternatives Considered**:
1. **Keep current structure, add wrapper**: Wrap the existing function - rejected because it would duplicate conversion logic
2. **Create new function**: Add a parallel function - rejected because it would create maintenance burden
3. **Modify existing function**: Change return type - **CHOSEN** - cleanest approach, maintains single source of truth

**Implementation Approach**:
- Change `record_batch_to_protobuf_bytes` signature to return `ProtobufConversionResult`
- Modify conversion loop to catch errors per row instead of returning immediately
- Update all call sites to handle the new return type
- Ensure backward compatibility by providing helper methods if needed

---

### Q2: How should we handle per-row errors during transmission (after conversion)?

**Decision**: Track per-row transmission errors separately from conversion errors, then merge them in `TransmissionResult`.

**Rationale**:
- Conversion errors occur during Arrow-to-Protobuf conversion (serialization)
- Transmission errors occur during network transmission (sending to Zerobus)
- Both types of errors need to be tracked per-row
- The transmission loop already tracks `failed_at_idx` (line 695, 735) but doesn't collect all failures
- We need to modify the transmission loop to continue processing remaining rows after a failure (for non-fatal errors)

**Alternatives Considered**:
1. **Stop on first transmission error**: Current behavior - rejected because it doesn't support partial success
2. **Continue on all errors**: Continue processing all rows - **CHOSEN** - enables partial success
3. **Stop on fatal errors only**: Distinguish fatal vs non-fatal - considered but adds complexity; can be future enhancement

**Implementation Approach**:
- Modify `send_batch_internal` to collect per-row transmission errors
- Continue processing remaining rows after a row fails (for retryable errors)
- Track both successful and failed row indices during transmission
- Merge conversion errors and transmission errors in final `TransmissionResult`

---

### Q3: How should per-row errors interact with retry logic?

**Decision**: Per-row errors should be preserved across retry attempts, with retry logic applying to batch-level transient failures.

**Rationale**:
- Retry logic currently handles transient failures (connection errors, stream closures)
- Per-row errors (conversion errors, validation errors) are typically non-retryable
- We should retry only failed rows that had transient errors, not rows with permanent errors
- Batch-level errors (authentication, connection) should still trigger retries at the batch level

**Alternatives Considered**:
1. **Retry entire batch**: Current behavior - rejected because it doesn't leverage per-row information
2. **Retry only failed rows**: Retry individual rows - **CHOSEN** - more efficient, enables partial success
3. **No retries for per-row errors**: Don't retry any per-row errors - rejected because some transmission errors are transient

**Implementation Approach**:
- Distinguish between retryable and non-retryable per-row errors
- For retryable errors (transmission failures), retry only those specific rows
- For non-retryable errors (conversion errors), don't retry
- Preserve error information across retry attempts

---

### Q4: How should we structure the new TransmissionResult fields?

**Decision**: Add optional fields to `TransmissionResult` for backward compatibility:
- `failed_rows: Option<Vec<(usize, ZerobusError)>>` - (row_index, error)
- `successful_rows: Option<Vec<usize>>` - row indices
- `total_rows: usize` - total rows in batch
- `successful_count: usize` - count of successful rows
- `failed_count: usize` - count of failed rows

**Rationale**:
- Optional fields maintain backward compatibility (existing code ignores them)
- Row indices (usize) are efficient and match Arrow's 0-based indexing
- Separate counts enable quick checks without iterating vectors
- `success` field remains for batch-level status (true if ANY rows succeeded)

**Alternatives Considered**:
1. **Separate struct**: Create `TransmissionResultWithRowErrors` - rejected because it breaks API consistency
2. **Required fields**: Make all fields required - rejected because it breaks backward compatibility
3. **Optional fields**: Add optional fields - **CHOSEN** - maintains compatibility while enabling new functionality

**Implementation Approach**:
- Add new fields to `TransmissionResult` struct
- Initialize with `None` or empty vectors when not applicable
- Update Python bindings to expose new fields
- Ensure serialization works correctly for both Rust and Python

---

### Q5: How should we handle batch-level errors vs row-level errors?

**Decision**: Batch-level errors (authentication, connection failures before row processing) should be reported at batch level. Row-level errors (conversion, transmission) should be reported per-row.

**Rationale**:
- Batch-level errors affect the entire batch and prevent any row processing
- Row-level errors affect individual rows and allow partial success
- Clear distinction helps users understand error scope
- `error` field remains for batch-level errors, `failed_rows` for row-level errors

**Alternatives Considered**:
1. **All errors per-row**: Convert all errors to per-row - rejected because batch-level errors don't apply to specific rows
2. **Separate error types**: Distinguish in error enum - **CHOSEN** - clear and maintainable
3. **Error hierarchy**: Nested error structure - rejected as over-engineering

**Implementation Approach**:
- Keep `error: Option<ZerobusError>` for batch-level errors
- Use `failed_rows: Option<Vec<(usize, ZerobusError)>>` for row-level errors
- Document when each field is populated
- Ensure error messages clearly indicate error scope

---

### Q6: How should we handle performance for large batches with many failures?

**Decision**: Use efficient data structures (Vec with pre-allocated capacity) and only collect errors when needed.

**Rationale**:
- Vec allocation is O(1) amortized, efficient for typical batch sizes
- Pre-allocating capacity reduces reallocations
- Only collect errors when rows fail (no overhead for successful batches)
- Memory overhead is bounded: O(failed_rows) not O(total_rows)

**Alternatives Considered**:
1. **Lazy collection**: Only collect on error - **CHOSEN** - minimal overhead for success cases
2. **Always collect**: Collect all row statuses - rejected because of unnecessary overhead
3. **Sampling**: Collect only sample of errors - rejected because it loses information

**Implementation Approach**:
- Pre-allocate vectors with estimated capacity when errors occur
- Use `Vec::with_capacity` for known sizes
- Avoid cloning errors unnecessarily (use references where possible)
- Benchmark performance impact

---

### Q7: How should Python bindings expose per-row error information?

**Decision**: Expose per-row error fields as Python-native types (list of tuples, list of ints) with proper type hints.

**Rationale**:
- Python users expect native Python types, not Rust-specific types
- Tuples `(int, ZerobusError)` map naturally to Python `(int, str)` or custom error objects
- List of ints for successful rows is intuitive
- Type hints enable better IDE support and documentation

**Alternatives Considered**:
1. **Custom Python classes**: Create Python classes - considered but adds complexity
2. **Dict structures**: Use dicts instead of tuples - considered but tuples are more efficient
3. **Native Python types**: Use lists and tuples - **CHOSEN** - simplest and most Pythonic

**Implementation Approach**:
- Convert Rust `Vec<(usize, ZerobusError)>` to Python `List[Tuple[int, str]]` or similar
- Convert Rust `Vec<usize>` to Python `List[int]`
- Ensure error messages are properly converted to Python strings
- Add Python docstrings with type hints

---

## Key Technical Decisions Summary

1. **Conversion**: Modify `record_batch_to_protobuf_bytes` to return `ProtobufConversionResult` with per-row errors
2. **Transmission**: Track per-row transmission errors and continue processing remaining rows
3. **Retry Logic**: Preserve per-row errors across retries, retry only failed rows with transient errors
4. **Data Structure**: Add optional fields to `TransmissionResult` for backward compatibility
5. **Error Types**: Distinguish batch-level vs row-level errors clearly
6. **Performance**: Use efficient Vec structures, only collect errors when needed
7. **Python Bindings**: Expose as native Python types (lists, tuples)

## Open Questions Resolved

✅ All research questions resolved - no NEEDS CLARIFICATION markers remain

## References

- Existing code: `src/wrapper/conversion.rs` (ProtobufConversionResult struct)
- Existing code: `src/wrapper/mod.rs` (TransmissionResult struct, send_batch methods)
- Existing code: `src/wrapper/zerobus.rs` (transmission loop with failed_at_idx tracking)
- Existing code: `src/error.rs` (ZerobusError enum)
- Existing code: `src/python/bindings.rs` (Python bindings patterns)

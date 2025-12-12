# Research: Debug Output Configuration Fixes

**Feature**: 005-debug-config-fixes  
**Date**: 2025-12-12  
**Status**: Complete

## Research Tasks

### 1. Separate Arrow/Protobuf Debug Flags Implementation

**Task**: Research best approach for adding separate configuration flags while maintaining backward compatibility.

**Decision**: Add two new boolean fields (`debug_arrow_enabled` and `debug_protobuf_enabled`) to `WrapperConfiguration` struct, with backward compatibility logic that sets both to `true` if the legacy `debug_enabled` flag is set.

**Rationale**: 
- Minimal code changes required
- Clear separation of concerns
- Maintains existing API surface for backward compatibility
- Follows existing configuration pattern in codebase

**Alternatives Considered**:
- **Option A**: Enum-based approach (`DebugOutputFormat::None | Arrow | Protobuf | Both`)
  - **Rejected**: More complex, requires enum handling throughout codebase, less intuitive for users
- **Option B**: Remove `debug_enabled` entirely, require explicit flags
  - **Rejected**: Breaks backward compatibility, violates FR-007 requirement

**Implementation Notes**:
- Configuration loaders (YAML, env vars) will check for new flags first, fall back to `debug_enabled` if not present
- `DebugWriter` will check flags independently before writing Arrow/Protobuf files
- Python bindings will expose both flags with same semantics as Rust API

### 2. File Rotation Timestamp Recursion Fix

**Task**: Research approach to prevent recursive timestamp appending in rotated file names.

**Decision**: Extract the original base filename (without any timestamp suffix) before generating rotated paths. Use regex pattern matching to detect and strip timestamp patterns (`_YYYYMMDD_HHMMSS`) from filenames.

**Rationale**:
- Prevents filename length issues
- Maintains readable file names
- Works with existing rotation logic
- Minimal performance impact (single regex match per rotation)

**Alternatives Considered**:
- **Option A**: Store base filename separately in `DebugWriter` struct
  - **Rejected**: Requires additional state management, more complex
- **Option B**: Use sequential numbering instead of timestamps
  - **Rejected**: Less informative, doesn't solve the core issue, changes user experience
- **Option C**: Truncate filenames if they exceed limits
  - **Rejected**: Loses information, doesn't prevent recursion issue

**Implementation Notes**:
- Timestamp pattern: `_(\d{8}_\d{6})` (matches `_YYYYMMDD_HHMMSS`)
- Extract base filename by removing timestamp suffix before appending new timestamp
- Apply fix to both `generate_rotated_path()` in `debug.rs` and `rotate_file_if_needed()` in `file_rotation.rs`
- Handle edge cases: filenames with multiple underscores, filenames without timestamps

### 3. Configuration Loading Strategy

**Task**: Determine how to handle configuration precedence for new flags vs. legacy flag.

**Decision**: Use explicit flag checking with fallback: if `debug_arrow_enabled` or `debug_protobuf_enabled` are explicitly set (not default), use them. Otherwise, if `debug_enabled` is set, enable both formats.

**Rationale**:
- Clear precedence rules
- Maintains backward compatibility
- Allows gradual migration

**Implementation Pattern**:
```rust
let arrow_enabled = config.debug_arrow_enabled 
    || (config.debug_enabled && !config.debug_arrow_enabled_set_explicitly);
```

**Alternatives Considered**:
- **Option A**: Always prefer new flags, ignore `debug_enabled` if new flags exist
  - **Rejected**: More complex logic, harder to reason about
- **Option B**: Deprecate `debug_enabled` immediately
  - **Rejected**: Violates backward compatibility requirement

### 4. Python Bindings API Design

**Task**: Determine how to expose new flags in Python bindings.

**Decision**: Add `debug_arrow_enabled` and `debug_protobuf_enabled` as optional keyword arguments to `PyWrapperConfiguration.__init__()`, with default `None` to maintain backward compatibility.

**Rationale**:
- Matches Rust API semantics
- Optional parameters allow gradual adoption
- Default `None` triggers backward compatibility logic

**Implementation Notes**:
- Python signature: `debug_arrow_enabled=None, debug_protobuf_enabled=None`
- If both are `None` and `debug_enabled=True`, enable both formats
- If either is explicitly set, use that value

### 5. File Rotation Testing Strategy

**Task**: Determine how to test recursive timestamp prevention.

**Decision**: Create test that simulates 100+ file rotations and verifies:
1. Each rotated file contains exactly one timestamp
2. Filenames never exceed 255 characters
3. Base filename is preserved across rotations

**Rationale**:
- Directly tests the bug fix
- Validates success criteria SC-002 and SC-003
- Catches edge cases early

**Test Approach**:
- Use `tempfile` crate for isolated test directories
- Simulate rotations by calling `rotate_file_if_needed()` repeatedly
- Assert filename patterns match expected format

## Technical Decisions Summary

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Add separate boolean flags | Simple, clear, backward compatible | Low - extends existing struct |
| Regex-based timestamp extraction | Prevents recursion, minimal changes | Low - single function modification |
| Explicit flag precedence | Clear behavior, maintainable | Low - configuration logic only |
| Optional Python parameters | Matches Rust API, gradual adoption | Low - extends existing bindings |
| Comprehensive rotation tests | Validates fix, catches regressions | Medium - new test cases required |

## Dependencies & Integration Points

### Modified Files
1. `src/config/types.rs` - Add `debug_arrow_enabled`, `debug_protobuf_enabled` fields
2. `src/config/loader.rs` - Update YAML/env var loading logic
3. `src/wrapper/debug.rs` - Fix `generate_rotated_path()`, add conditional writing
4. `src/wrapper/mod.rs` - Update debug writer initialization logic
5. `src/python/bindings.rs` - Add Python bindings for new flags
6. `src/utils/file_rotation.rs` - Fix timestamp extraction in rotation logic

### Test Files
1. `tests/unit/config/test_loader.rs` - Test new flag loading
2. `tests/unit/wrapper/test_debug.rs` - Test rotation fix
3. `tests/integration/test_debug_files.rs` - Test separate flag behavior
4. `tests/contract/test_rust_api_contract.rs` - Verify API contract

### No Breaking Changes
- Existing `debug_enabled` flag continues to work
- Python API remains backward compatible
- No changes to file formats or directory structure

## Open Questions Resolved

- ✅ **Q1**: How to handle backward compatibility?
  - **A**: Check new flags first, fall back to `debug_enabled` if not set
  
- ✅ **Q2**: Should descriptor files be written if only one format is enabled?
  - **A**: Yes, write descriptors if either Arrow or Protobuf debug is enabled (FR-009)
  
- ✅ **Q3**: What happens if both flags are disabled but `debug_output_dir` is set?
  - **A**: Valid configuration - no files written (edge case documented)

### 6. File Retention Implementation

**Task**: Research best approach for implementing automatic file retention with cleanup.

**Decision**: Implement file retention cleanup immediately after file rotation when retention limit is exceeded. Use directory scanning to find rotated files, parse timestamps/sequential numbers from filenames to determine order, and delete oldest files. Apply retention limits independently per file type (Arrow and Protobuf).

**Rationale**:
- Immediate cleanup prevents disk space accumulation
- Independent limits per type provide predictable disk usage
- Parsing timestamps from filenames is reliable and doesn't require filesystem metadata
- Bounded by retention limit (default 10), so cleanup is O(n) where n is small

**Alternatives Considered**:
- **Option A**: Background cleanup task (periodic scanning)
  - **Rejected**: More complex, requires background thread/task management, less predictable timing
- **Option B**: Combined limit across both file types
  - **Rejected**: Less predictable, harder to reason about disk usage per format
- **Option C**: Use filesystem modification time for ordering
  - **Rejected**: Less portable, requires metadata access, timestamp in filename is more reliable

**Implementation Notes**:
- Add `debug_max_files_retained: Option<usize>` to `WrapperConfiguration` (default: Some(10), None = unlimited)
- After rotation, scan directory for rotated files matching pattern
- Parse timestamp or sequential number from filename to determine order
- If count exceeds limit, delete oldest files until limit satisfied
- Only count rotated/closed files (exclude currently active file)
- Log errors but don't fail rotation if deletion fails
- Apply independently to Arrow and Protobuf directories

### 7. File Deletion Ordering Strategy

**Task**: Determine how to identify oldest files for deletion when retention limit exceeded.

**Decision**: Parse timestamp or sequential number from filename to determine file age. For timestamp-based names, extract `_YYYYMMDD_HHMMSS` pattern. For sequential-based names, extract numeric suffix. Sort files by parsed value and delete oldest.

**Rationale**:
- Filenames contain ordering information (timestamp or sequence number)
- More reliable than filesystem metadata (portable across systems)
- Efficient for small number of files (bounded by retention limit)
- Works with both timestamp and sequential naming schemes

**Implementation Pattern**:
```rust
// Parse timestamp from filename: table_20241212_120000.proto
let timestamp = extract_timestamp(&filename)?;
// Or parse sequential number: table_0001.proto
let sequence = extract_sequence(&filename)?;
// Sort by timestamp/sequence, delete oldest
```

## Technical Decisions Summary

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Add separate boolean flags | Simple, clear, backward compatible | Low - extends existing struct |
| Regex-based timestamp extraction | Prevents recursion, minimal changes | Low - single function modification |
| Explicit flag precedence | Clear behavior, maintainable | Low - configuration logic only |
| Optional Python parameters | Matches Rust API, gradual adoption | Low - extends existing bindings |
| Comprehensive rotation tests | Validates fix, catches regressions | Medium - new test cases required |
| Immediate retention cleanup | Predictable disk usage, simple implementation | Medium - requires directory scanning |
| Independent limits per type | Predictable per-format disk usage | Low - separate cleanup per directory |
| Filename-based ordering | Portable, reliable, efficient | Low - parsing logic required |

## Dependencies & Integration Points

### Modified Files
1. `src/config/types.rs` - Add `debug_arrow_enabled`, `debug_protobuf_enabled`, `debug_max_files_retained` fields
2. `src/config/loader.rs` - Update YAML/env var loading logic
3. `src/wrapper/debug.rs` - Fix `generate_rotated_path()`, add conditional writing, add retention cleanup
4. `src/wrapper/mod.rs` - Update debug writer initialization logic
5. `src/python/bindings.rs` - Add Python bindings for new flags and retention
6. `src/utils/file_rotation.rs` - Fix timestamp extraction in rotation logic

### Test Files
1. `tests/unit/config/test_loader.rs` - Test new flag and retention loading
2. `tests/unit/wrapper/test_debug.rs` - Test rotation fix and retention cleanup
3. `tests/integration/test_debug_files.rs` - Test separate flag behavior and retention
4. `tests/contract/test_rust_api_contract.rs` - Verify API contract

### No Breaking Changes
- Existing `debug_enabled` flag continues to work
- Python API remains backward compatible
- No changes to file formats or directory structure
- File retention defaults to 10 files (reasonable default)

## Open Questions Resolved

- ✅ **Q1**: How to handle backward compatibility?
  - **A**: Check new flags first, fall back to `debug_enabled` if not set
  
- ✅ **Q2**: Should descriptor files be written if only one format is enabled?
  - **A**: Yes, write descriptors if either Arrow or Protobuf debug is enabled (FR-009)
  
- ✅ **Q3**: What happens if both flags are disabled but `debug_output_dir` is set?
  - **A**: Valid configuration - no files written (edge case documented)

- ✅ **Q4**: How should file retention be implemented?
  - **A**: Immediate cleanup after rotation, independent limits per type, parse timestamps from filenames

- ✅ **Q5**: What happens when file deletion fails?
  - **A**: Log error and continue - deletion failure should not block rotation

## Next Steps

1. Implement configuration struct changes (flags + retention)
2. Update configuration loaders
3. Fix file rotation logic
4. Update debug writer conditional logic
5. Implement file retention cleanup logic
6. Add Python bindings
7. Write comprehensive tests
8. Update documentation

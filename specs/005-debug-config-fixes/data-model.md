# Data Model: Debug Output Configuration Fixes

**Feature**: 005-debug-config-fixes  
**Date**: 2025-12-12

## Entities

### DebugConfiguration

Represents the debug output settings for the Zerobus wrapper, including separate flags for Arrow and Protobuf output formats.

**Fields**:
- `debug_arrow_enabled: bool` - Enable/disable Arrow debug file output (default: false)
- `debug_protobuf_enabled: bool` - Enable/disable Protobuf debug file output (default: false)
- `debug_enabled: bool` - Legacy flag for backward compatibility (default: false)
  - If set to `true` and new flags are not explicitly set, enables both Arrow and Protobuf
- `debug_output_dir: Option<PathBuf>` - Output directory for debug files (required if either format enabled)
- `debug_flush_interval_secs: u64` - Flush interval in seconds (default: 5)
- `debug_max_file_size: Option<u64>` - Maximum file size in bytes before rotation (optional)
- `debug_max_files_retained: Option<usize>` - Maximum number of rotated files to retain per type (default: Some(10), None = unlimited)

**Relationships**:
- Part of `WrapperConfiguration` struct
- Used by `DebugWriter` to determine which formats to write

**Validation Rules**:
- If `debug_arrow_enabled` or `debug_protobuf_enabled` is `true`, `debug_output_dir` must be `Some(path)`
- If `debug_enabled` is `true` and new flags are not set, both formats are enabled
- `debug_flush_interval_secs` must be > 0
- `debug_max_file_size` must be > 0 if `Some`

**State Transitions**:
- Initialization: Flags default to `false`, `debug_output_dir` is `None`, `debug_max_files_retained` defaults to `Some(10)`
- Configuration loading: Flags and retention limit set from YAML/env vars/programmatic API
- Backward compatibility: If `debug_enabled=true` and new flags not set, both new flags set to `true`
- File retention: When rotation occurs and file count exceeds limit, oldest files deleted immediately

### FileRotationState

Tracks the current file path and base filename for rotation, preventing recursive timestamp appending.

**Fields**:
- `base_file_path: PathBuf` - Original file path without timestamp suffix
- `current_file_path: PathBuf` - Current file path (may include timestamp after rotation)
- `rotation_count: usize` - Number of times file has been rotated (for tracking/debugging)

**Relationships**:
- Used by `DebugWriter` for both Arrow and Protobuf file rotation
- Managed internally by `DebugWriter` struct

**Validation Rules**:
- `base_file_path` must not contain timestamp pattern (`_YYYYMMDD_HHMMSS`)
- `current_file_path` may contain at most one timestamp suffix
- Rotation count increments on each rotation

**State Transitions**:
- Initialization: `base_file_path` = original filename, `current_file_path` = same, `rotation_count` = 0
- Rotation: Extract base from `current_file_path`, append new timestamp, increment count
- Reset: Not applicable (state persists for file lifetime)

### BaseFileName

The original file name without any timestamp suffixes, used as the foundation for generating rotated file names.

**Fields**:
- `name: String` - Base filename without extension
- `extension: String` - File extension (e.g., "arrows", "proto")
- `sanitized_table_name: String` - Original table name with filesystem-safe characters

**Relationships**:
- Derived from `table_name` in `WrapperConfiguration`
- Used by `generate_rotated_path()` to create rotated filenames

**Validation Rules**:
- Must not contain timestamp pattern (`_YYYYMMDD_HHMMSS`)
- Must be filesystem-safe (no invalid characters)
- Extension must be valid (non-empty, no dots)

**Extraction Logic**:
1. Start with original filename (e.g., `table_20241212_120000.proto`)
2. Remove extension to get stem (e.g., `table_20241212_120000`)
3. Match timestamp pattern: `_(\d{8}_\d{6})$` (end of string)
4. If match found, remove timestamp suffix (e.g., `table`)
5. Return base name + extension

## Data Flow

### Configuration Loading Flow

```
User Input (YAML/Env/API)
    ↓
Configuration Loader
    ↓
WrapperConfiguration
    ├── debug_arrow_enabled: bool
    ├── debug_protobuf_enabled: bool
    └── debug_enabled: bool (legacy)
    ↓
Backward Compatibility Check
    ├── If debug_enabled=true AND new flags not set
    │   └── Set both new flags to true
    └── Otherwise use explicit flag values
    ↓
DebugWriter Initialization
    ├── If debug_arrow_enabled: Initialize Arrow writer
    └── If debug_protobuf_enabled: Initialize Protobuf writer
```

### File Rotation Flow

```
Current File Path (may have timestamp)
    ↓
Extract Base Filename
    ├── Remove extension
    ├── Check for timestamp pattern at end
    └── Remove timestamp if found
    ↓
Generate Rotated Path
    ├── Base filename (no timestamp)
    ├── Append new timestamp: _YYYYMMDD_HHMMSS
    └── Append extension
    ↓
New Rotated File Path (exactly one timestamp)
    ↓
Check Retention Limit
    ├── Count rotated files in directory
    ├── If count > limit:
    │   ├── Parse timestamps/sequences from filenames
    │   ├── Sort by age (oldest first)
    │   └── Delete oldest files until count = limit
    └── Continue with new file
```

## Constraints

### Filename Length Constraints
- Maximum filename length: 255 characters (POSIX standard)
- Base filename + timestamp + extension must not exceed limit
- If limit would be exceeded, truncate base filename (preserve extension and timestamp)

### Timestamp Format Constraints
- Format: `_YYYYMMDD_HHMMSS` (17 characters including underscore)
- Must match pattern: `_(\d{8}_\d{6})`
- Only appended at end of filename (before extension)

### Configuration Constraints
- At least one debug format must be enabled if `debug_output_dir` is set (or allow both disabled)
- `debug_output_dir` must be valid filesystem path
- Directory must be writable by process

## Examples

### Example 1: Separate Flags Configuration

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_arrow_enabled(true)
    .with_debug_protobuf_enabled(false)
    .with_debug_output(PathBuf::from("/tmp/debug"));
```

**Result**: Only Arrow files written, Protobuf files not written.

### Example 2: Backward Compatibility

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_output(PathBuf::from("/tmp/debug"));
// debug_enabled defaults to false, so no debug files written

// OR legacy code:
let config = WrapperConfiguration::new(...)
    .with_debug_output(PathBuf::from("/tmp/debug"));
config.debug_enabled = true; // Legacy flag
```

**Result**: Both Arrow and Protobuf files written (backward compatibility).

### Example 3: File Rotation

**Initial**: `table.proto`  
**After 1st rotation**: `table_20241212_120000.proto`  
**After 2nd rotation**: `table_20241212_120100.proto` (not `table_20241212_120000_20241212_120100.proto`)

**Base extraction**:
- Input: `table_20241212_120000.proto`
- Remove extension: `table_20241212_120000`
- Match timestamp: `_20241212_120000` ✓
- Remove timestamp: `table`
- New rotation: `table_20241212_120100.proto` ✓

### Example 4: File Retention

**Initial state**: 10 rotated files exist (`table_20241212_120000.proto` through `table_20241212_120900.proto`)  
**Rotation occurs**: New file `table_20241212_121000.proto` created  
**Retention check**: 11 files > limit of 10  
**Cleanup**: Delete oldest file `table_20241212_120000.proto`  
**Result**: 10 files remain (`table_20241212_120100.proto` through `table_20241212_121000.proto`)

# Quickstart: Debug Output Configuration Fixes

**Feature**: 005-debug-config-fixes  
**Date**: 2025-12-12

## Overview

This feature adds two improvements to debug output configuration:

1. **Separate Arrow/Protobuf Debug Flags**: Enable Arrow and Protobuf debug output independently
2. **Fixed File Rotation**: Prevents "File name too long" errors from recursive timestamp appending

## Quick Start

### Rust Example: Enable Only Arrow Debug

```rust
use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper};
use std::path::PathBuf;

let config = WrapperConfiguration::new(
    "https://example.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_debug_arrow_enabled(true)      // Enable Arrow debug files
.with_debug_protobuf_enabled(false)  // Disable Protobuf debug files
.with_debug_output(PathBuf::from("/tmp/debug"));

let wrapper = ZerobusWrapper::new(config).await?;
// Only Arrow files (.arrows) will be written to /tmp/debug/zerobus/arrow/
```

### Python Example: Enable Only Protobuf Debug

```python
from arrow_zerobus_sdk_wrapper import PyWrapperConfiguration, ZerobusWrapper

config = PyWrapperConfiguration(
    endpoint="https://example.cloud.databricks.com",
    table_name="my_table",
    debug_arrow_enabled=False,      # Disable Arrow debug files
    debug_protobuf_enabled=True,    # Enable Protobuf debug files
    debug_output_dir="/tmp/debug"
)

wrapper = ZerobusWrapper(config)
# Only Protobuf files (.proto) will be written to /tmp/debug/zerobus/proto/
```

### Backward Compatible Example

```rust
// Legacy code still works - enables both formats
let config = WrapperConfiguration::new(...)
    .with_debug_output(PathBuf::from("/tmp/debug"));
config.debug_enabled = true; // Legacy flag
```

```python
# Legacy Python code still works
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_enabled=True,  # Enables both Arrow and Protobuf
    debug_output_dir="/tmp/debug"
)
```

## Configuration Methods

### Method 1: Programmatic API (Rust)

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_arrow_enabled(true)
    .with_debug_protobuf_enabled(false)
    .with_debug_output(PathBuf::from("/tmp/debug"));
```

### Method 2: YAML Configuration

```yaml
debug:
  arrow_enabled: true
  protobuf_enabled: false
  output_dir: "/tmp/debug"
  flush_interval_secs: 5
  max_file_size: 10485760
```

### Method 3: Environment Variables

```bash
export DEBUG_ARROW_ENABLED=true
export DEBUG_PROTOBUF_ENABLED=false
export DEBUG_OUTPUT_DIR=/tmp/debug
export DEBUG_FLUSH_INTERVAL_SECS=5
```

## File Rotation Fix

The file rotation bug is automatically fixed. No code changes required.

**Before** (buggy behavior):
- `table.proto` → `table_20241212_120000.proto` → `table_20241212_120000_20241212_120100.proto` ❌

**After** (fixed behavior):
- `table.proto` → `table_20241212_120000.proto` → `table_20241212_120100.proto` ✅

Each rotated file contains exactly one timestamp, preventing filename length issues.

## File Retention

The system automatically maintains a configurable number of rotated debug files (default: 10 per type). When the limit is exceeded, the oldest files are automatically deleted.

### Rust Example: Configure File Retention

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_arrow_enabled(true)
    .with_debug_protobuf_enabled(true)
    .with_debug_output(PathBuf::from("/tmp/debug"))
    .with_debug_max_files_retained(Some(10)); // Keep last 10 files per type
```

### Python Example: Configure File Retention

```python
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_protobuf_enabled=True,
    debug_output_dir="/tmp/debug",
    debug_max_files_retained=10  # Keep last 10 files per type
)
```

### Unlimited Retention

To disable automatic cleanup (unlimited retention):

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_max_files_retained(None); // Unlimited
```

```python
config = PyWrapperConfiguration(
    ...,
    debug_max_files_retained=None  # Unlimited
)
```

## Common Use Cases

### Use Case 1: Debug Arrow Data Only

**Scenario**: You want to inspect Arrow RecordBatch data without generating Protobuf files.

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_arrow_enabled(true)
    .with_debug_protobuf_enabled(false)
    .with_debug_output(PathBuf::from("/tmp/debug"));
```

**Result**: Only `/tmp/debug/zerobus/arrow/table.arrows` files are created.

### Use Case 2: Debug Protobuf Serialization Only

**Scenario**: You want to inspect Protobuf serialization without Arrow files.

```python
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=False,
    debug_protobuf_enabled=True,
    debug_output_dir="/tmp/debug"
)
```

**Result**: Only `/tmp/debug/zerobus/proto/table.proto` files are created.

### Use Case 3: Long-Running Process with File Rotation

**Scenario**: You have a long-running process that rotates files frequently.

**Before**: Files would accumulate timestamps: `table_20241212_120000_20241212_120100_20241212_120200.proto` (eventually causing "File name too long" errors)

**After**: Files rotate cleanly: `table_20241212_120000.proto` → `table_20241212_120100.proto` → `table_20241212_120200.proto` (no recursion)

### Use Case 4: Bounded Disk Usage with File Retention

**Scenario**: You want to prevent unlimited disk space consumption from debug files.

**Configuration**:
```rust
let config = WrapperConfiguration::new(...)
    .with_debug_arrow_enabled(true)
    .with_debug_output(PathBuf::from("/tmp/debug"))
    .with_debug_max_files_retained(Some(10)); // Keep last 10 files
```

**Result**: System automatically maintains exactly 10 rotated Arrow files. When the 11th file is created, the oldest file is deleted, keeping disk usage bounded.

## Migration Guide

### From Legacy `debug_enabled` Flag

**Step 1**: Identify which debug formats you need
- If you only need Arrow: Set `debug_arrow_enabled=true`, `debug_protobuf_enabled=false`
- If you only need Protobuf: Set `debug_arrow_enabled=false`, `debug_protobuf_enabled=true`
- If you need both: Set both flags to `true` (or continue using `debug_enabled=true`)

**Step 2**: Update configuration code

**Before**:
```rust
config.debug_enabled = true;
```

**After**:
```rust
config.debug_arrow_enabled = true;
config.debug_protobuf_enabled = true; // or false, depending on needs
```

**Step 3**: Test and verify
- Verify only intended debug files are created
- Check file rotation works correctly (no recursive timestamps)
- Verify file retention cleanup works (oldest files deleted when limit exceeded)

## Troubleshooting

### Issue: Debug files not being created

**Check**:
1. Is `debug_output_dir` set?
2. Is at least one debug flag (`debug_arrow_enabled` or `debug_protobuf_enabled`) set to `true`?
3. Does the output directory exist and is writable?

### Issue: "File name too long" errors

**Solution**: This should be fixed automatically. If you still see this error:
1. Check that you're using the latest version with the fix
2. Verify rotated files contain only one timestamp
3. Report as a bug if issue persists

### Issue: Both formats enabled when only one should be

**Check**:
1. Are you using the legacy `debug_enabled` flag? (It enables both formats)
2. Are both `debug_arrow_enabled` and `debug_protobuf_enabled` explicitly set?
3. Check configuration precedence: explicit flags override legacy flag

## See Also

- [Rust API Contract](./contracts/rust-api.md) - Detailed Rust API documentation
- [Python API Contract](./contracts/python-api.md) - Detailed Python API documentation
- [Data Model](./data-model.md) - Data structures and relationships
- [Specification](./spec.md) - Full feature specification

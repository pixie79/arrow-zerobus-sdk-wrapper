# Rust API Contract: Debug Output Configuration Fixes

**Feature**: 005-debug-config-fixes  
**Date**: 2025-12-12

## Configuration API Changes

### WrapperConfiguration Struct

**Location**: `src/config/types.rs`

**New Fields**:
```rust
pub struct WrapperConfiguration {
    // ... existing fields ...
    
    /// Enable/disable Arrow debug file output (default: false)
    /// When true, Arrow debug files (.arrows) are written to debug_output_dir
    pub debug_arrow_enabled: bool,
    
    /// Enable/disable Protobuf debug file output (default: false)
    /// When true, Protobuf debug files (.proto) are written to debug_output_dir
    pub debug_protobuf_enabled: bool,
    
    /// Legacy flag for backward compatibility (default: false)
    /// If true and new flags are not explicitly set, enables both Arrow and Protobuf
    /// @deprecated Use debug_arrow_enabled and debug_protobuf_enabled instead
    pub debug_enabled: bool,
    
    /// Maximum number of rotated debug files to retain per type (default: Some(10))
    /// When Some(n), keeps last n rotated files, automatically deleting oldest when limit exceeded
    /// When None, unlimited retention (no automatic cleanup)
    pub debug_max_files_retained: Option<usize>,
    
    // ... existing fields ...
}
```

### Builder Methods

**Location**: `src/config/types.rs`

**New Methods**:
```rust
impl WrapperConfiguration {
    /// Set Arrow debug output enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - If true, Arrow debug files will be written
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_debug_arrow_enabled(mut self, enabled: bool) -> Self {
        self.debug_arrow_enabled = enabled;
        self
    }
    
    /// Set Protobuf debug output enabled
    ///
    /// # Arguments
    ///
    /// * `enabled` - If true, Protobuf debug files will be written
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_debug_protobuf_enabled(mut self, enabled: bool) -> Self {
        self.debug_protobuf_enabled = enabled;
        self
    }
    
    /// Set debug file retention limit
    ///
    /// # Arguments
    ///
    /// * `max_files` - Maximum number of rotated files to retain per type (default: Some(10), None = unlimited)
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_debug_max_files_retained(mut self, max_files: Option<usize>) -> Self {
        self.debug_max_files_retained = max_files;
        self
    }
    
    // Existing with_debug_output() method remains unchanged
}
```

### Configuration Loading

**Location**: `src/config/loader.rs`

**YAML Configuration**:
```yaml
debug:
  enabled: true  # Legacy flag (enables both if new flags not set)
  arrow_enabled: true   # New flag (optional)
  protobuf_enabled: false  # New flag (optional)
  output_dir: "/tmp/debug"
  flush_interval_secs: 5
  max_file_size: 10485760
  max_files_retained: 10  # New flag (optional, default: 10, 0 = unlimited)
```

**Environment Variables**:
```bash
DEBUG_ARROW_ENABLED=true
DEBUG_PROTOBUF_ENABLED=false
DEBUG_ENABLED=true  # Legacy (optional)
DEBUG_OUTPUT_DIR=/tmp/debug
DEBUG_FLUSH_INTERVAL_SECS=5
DEBUG_MAX_FILE_SIZE=10485760
```

**Precedence Rules**:
1. If `debug_arrow_enabled` or `debug_protobuf_enabled` are explicitly set, use those values
2. Otherwise, if `debug_enabled` is set to `true`, enable both formats
3. If all flags are `false` or unset, no debug output is written

### DebugWriter API

**Location**: `src/wrapper/debug.rs`

**No Public API Changes**: The `DebugWriter` struct and its methods remain unchanged. Internal logic is modified to check configuration flags before writing.

**Internal Behavior Changes**:
- `write_arrow()`: Only writes if `debug_arrow_enabled` is `true`
- `write_protobuf()`: Only writes if `debug_protobuf_enabled` is `true`
- `write_descriptor()`: Writes if either `debug_arrow_enabled` or `debug_protobuf_enabled` is `true`

### File Rotation API

**Location**: `src/wrapper/debug.rs`, `src/utils/file_rotation.rs`

**Changed Function**:
```rust
impl DebugWriter {
    /// Generate rotated file path with timestamp
    /// 
    /// Fixed to prevent recursive timestamp appending by extracting
    /// base filename without timestamps before appending new timestamp.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Current file path (may contain timestamp from previous rotation)
    ///
    /// # Returns
    ///
    /// New file path with single timestamp suffix
    fn generate_rotated_path(base_path: &std::path::Path) -> PathBuf {
        // Extract base filename without timestamp
        // Append new timestamp: _YYYYMMDD_HHMMSS
        // Return new path
    }
}
```

**Behavior**:
- Input: `table_20241212_120000.proto` (already rotated)
- Extract base: `table.proto` (remove timestamp)
- Append new timestamp: `table_20241212_120100.proto`
- Output: Single timestamp, no recursion

## Backward Compatibility

### Existing Code Compatibility

**Before** (still works):
```rust
let config = WrapperConfiguration::new(...)
    .with_debug_output(PathBuf::from("/tmp/debug"));
config.debug_enabled = true; // Legacy flag
```

**After** (new preferred way):
```rust
let config = WrapperConfiguration::new(...)
    .with_debug_arrow_enabled(true)
    .with_debug_protobuf_enabled(false)
    .with_debug_output(PathBuf::from("/tmp/debug"));
```

**Migration Path**:
1. Existing code using `debug_enabled` continues to work
2. New code should use `debug_arrow_enabled` and `debug_protobuf_enabled`
3. `debug_enabled` is deprecated but not removed

## Error Handling

**No New Error Types**: Existing `ZerobusError::ConfigurationError` is used for invalid configurations.

**Validation Errors**:
- If `debug_arrow_enabled` or `debug_protobuf_enabled` is `true` but `debug_output_dir` is `None`: ConfigurationError
- If `debug_output_dir` is invalid path: ConfigurationError
- If file rotation fails due to filename length: ConfigurationError (with message about truncation)

## Examples

### Example 1: Enable Only Arrow Debug

```rust
use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper};

let config = WrapperConfiguration::new(
    "https://example.com".to_string(),
    "my_table".to_string(),
)
.with_debug_arrow_enabled(true)
.with_debug_protobuf_enabled(false)
.with_debug_output(std::path::PathBuf::from("/tmp/debug"));

let wrapper = ZerobusWrapper::new(config).await?;
// Only Arrow files written, Protobuf files not written
```

### Example 2: Backward Compatible Configuration

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_output(PathBuf::from("/tmp/debug"));
// Set legacy flag
config.debug_enabled = true;
// Both formats enabled automatically
```

### Example 3: Environment Variable Configuration

```bash
export DEBUG_ARROW_ENABLED=true
export DEBUG_PROTOBUF_ENABLED=false
export DEBUG_OUTPUT_DIR=/tmp/debug
export DEBUG_MAX_FILES_RETAINED=10
```

```rust
let config = WrapperConfiguration::load_from_env()?;
// Arrow enabled, Protobuf disabled, retention limit: 10 files
```

### Example 4: File Retention Configuration

```rust
use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper};

let config = WrapperConfiguration::new(
    "https://example.com".to_string(),
    "my_table".to_string(),
)
.with_debug_arrow_enabled(true)
.with_debug_protobuf_enabled(true)
.with_debug_output(std::path::PathBuf::from("/tmp/debug"))
.with_debug_max_files_retained(Some(10)); // Keep last 10 files per type

let wrapper = ZerobusWrapper::new(config).await?;
// After 11th rotation, oldest file automatically deleted
```

### Example 5: Unlimited File Retention

```rust
let config = WrapperConfiguration::new(...)
    .with_debug_output(PathBuf::from("/tmp/debug"))
    .with_debug_max_files_retained(None); // Unlimited retention
```

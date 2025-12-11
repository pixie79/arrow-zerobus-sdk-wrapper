# Rust API Contract: Zerobus Writer Disabled Mode

**Feature**: 003-zerobus-writer-disabled  
**Date**: 2025-12-11  
**Version**: 0.1.0 (additive change)

## API Changes

This document describes the API changes for the Zerobus Writer Disabled Mode feature. All existing APIs remain unchanged; this feature adds a new configuration option.

### WrapperConfiguration (Modified)

Configuration structure updated to include writer disabled mode.

```rust
pub struct WrapperConfiguration {
    // ... existing fields ...
    
    /// Disable Zerobus SDK transmission while maintaining debug file output (default: false)
    /// 
    /// When `true`, the wrapper will skip all Zerobus SDK calls (initialization, 
    /// stream creation, data transmission) while still writing debug files (Arrow 
    /// and Protobuf) if debug output is enabled.
    /// 
    /// # Requirements
    /// - When `true`, `debug_enabled` must also be `true`
    /// - Credentials (`client_id`, `client_secret`) are optional when `true`
    /// 
    /// # Use Cases
    /// - Local development without network access
    /// - CI/CD testing without credentials
    /// - Performance testing of conversion logic
    pub zerobus_writer_disabled: bool,
}

impl WrapperConfiguration {
    // ... existing methods ...
    
    /// Set writer disabled mode
    /// 
    /// # Arguments
    /// * `disabled` - If `true`, disables Zerobus SDK transmission while maintaining debug output
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// use arrow_zerobus_sdk_wrapper::WrapperConfiguration;
    /// use std::path::PathBuf;
    /// 
    /// let config = WrapperConfiguration::new(
    ///     "https://workspace.cloud.databricks.com".to_string(),
    ///     "my_table".to_string(),
    /// )
    /// .with_debug_output(PathBuf::from("./debug_output"))
    /// .with_zerobus_writer_disabled(true);
    /// ```
    pub fn with_zerobus_writer_disabled(mut self, disabled: bool) -> Self;
    
    /// Validate configuration
    /// 
    /// # Errors
    /// Returns error if:
    /// - `zerobus_writer_disabled` is `true` but `debug_enabled` is `false`
    /// - Other existing validation failures
    pub fn validate(&self) -> Result<(), ZerobusError>;
}
```

### Validation Behavior

**New Validation Rule**:
- If `zerobus_writer_disabled` is `true` and `debug_enabled` is `false`, validation returns:
  ```rust
  Err(ZerobusError::ConfigurationError(
      "debug_enabled must be true when zerobus_writer_disabled is true".to_string()
  ))
  ```

**Modified Validation Rule**:
- When `zerobus_writer_disabled` is `true`, credential validation (`client_id`, `client_secret`, `unity_catalog_url`) is skipped (credentials become optional)

### ZerobusWrapper (Behavior Change)

No API changes, but behavior changes when `zerobus_writer_disabled` is `true`:

```rust
impl ZerobusWrapper {
    // ... existing methods unchanged ...
    
    /// Send a data batch to Zerobus
    /// 
    /// # Behavior When Writer Disabled
    /// 
    /// When `zerobus_writer_disabled` is `true`:
    /// - Debug files (Arrow and Protobuf) are written normally
    /// - Arrow-to-Protobuf conversion executes normally
    /// - Zerobus SDK initialization is skipped
    /// - Stream creation is skipped
    /// - Data transmission calls are skipped
    /// - Returns `TransmissionResult` with `success: true` if conversion succeeds
    /// - Returns `TransmissionResult` with `success: false` if conversion fails
    /// 
    /// # Arguments
    /// * `batch` - Arrow RecordBatch to send
    /// 
    /// # Returns
    /// TransmissionResult indicating success or failure
    /// 
    /// # Errors
    /// Returns error if conversion fails (when writer disabled) or transmission fails (when writer enabled)
    pub async fn send_batch(&self, batch: RecordBatch) -> Result<TransmissionResult, ZerobusError>;
}
```

### TransmissionResult (No Changes)

No structural changes. Behavior unchanged - returns success when writer is disabled and conversion succeeds.

```rust
pub struct TransmissionResult {
    pub success: bool,
    pub error: Option<ZerobusError>,
    pub attempts: u32,
    pub latency_ms: Option<u64>,
    pub batch_size_bytes: usize,
}
```

**Behavior Notes**:
- When writer is disabled: `success` is `true` if conversion succeeds, `false` if conversion fails
- `attempts` will be `1` (no retry logic executed when disabled)
- `latency_ms` reflects conversion time only (no network overhead)

## Error Types

### ZerobusError (No Changes)

No new error types. Existing `ConfigurationError` is used for validation failures.

## Examples

### Example 1: Local Development Without Network

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WrapperConfiguration::new(
        "https://workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    )
    .with_debug_output(PathBuf::from("./debug_output"))
    .with_zerobus_writer_disabled(true);  // Disable transmission
    
    let wrapper = ZerobusWrapper::new(config).await?;
    
    // Send batch - will write debug files but skip network calls
    let batch = create_test_batch()?;
    let result = wrapper.send_batch(batch).await?;
    
    // Result will be success (conversion succeeded)
    assert!(result.success);
    
    Ok(())
}
```

### Example 2: Configuration Validation

```rust
use arrow_zerobus_sdk_wrapper::WrapperConfiguration;

let config = WrapperConfiguration::new(
    "https://workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_zerobus_writer_disabled(true);  // But debug_enabled is false (default)

// Validation will fail
assert!(config.validate().is_err());
```

## Backward Compatibility

- ✅ All existing APIs remain unchanged
- ✅ Default value for `zerobus_writer_disabled` is `false` (existing behavior preserved)
- ✅ No breaking changes to existing code
- ✅ Optional feature - existing code continues to work without modification

## Migration Guide

No migration required. This is an additive feature. To use:

1. Add `.with_zerobus_writer_disabled(true)` to your configuration
2. Ensure `debug_enabled` is also `true` (or use `.with_debug_output()`)
3. Existing code continues to work without changes


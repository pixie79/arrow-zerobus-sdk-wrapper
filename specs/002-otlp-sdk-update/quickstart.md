# Quickstart: OTLP SDK Integration Update

**Feature**: 002-otlp-sdk-update  
**Date**: 2025-01-27

## Overview

This feature updates the observability implementation to use the SDK from otlp-rust-service, replacing manual construction of metrics and traces. All dead code is removed, and configuration is simplified to align with SDK requirements.

## Key Changes

1. **SDK-Based Observability**: Metrics and traces now use SDK methods instead of manual construction
2. **Configuration Update**: `OtlpConfig` replaced with `OtlpSdkConfig` (breaking change)
3. **Dead Code Removal**: Removed `new()`, `create_batch_metrics()`, `create_span_data()`, `convert_config()`
4. **Simplified Implementation**: Direct SDK usage, no conversion layers

## Migration Guide

### Configuration Changes

**Before**:
```rust
use arrow_zerobus_sdk_wrapper::{OtlpConfig, ObservabilityManager};

let otlp_config = OtlpConfig {
    endpoint: Some("https://otlp-endpoint".to_string()),
    log_level: "info".to_string(),
    extra: HashMap::new(),
};
```

**After**:
```rust
use arrow_zerobus_sdk_wrapper::{OtlpSdkConfig, ObservabilityManager};

let otlp_config = OtlpSdkConfig {
    endpoint: Some("https://otlp-endpoint".to_string()),
    output_dir: Some(PathBuf::from("/tmp/otlp")),
    write_interval_secs: 5,
    log_level: "info".to_string(),
};
```

### Initialization

**Before** (dead code - always returned None):
```rust
let manager = ObservabilityManager::new(Some(otlp_config));
```

**After** (unchanged, but config type changed):
```rust
let manager = ObservabilityManager::new_async(Some(otlp_config)).await;
```

### Usage

**No changes needed** - public API methods have same signatures:
- `record_batch_sent()` - Same signature, now uses SDK internally
- `start_send_batch_span()` - Same signature, now uses SDK internally
- `flush()` - Unchanged
- `shutdown()` - Unchanged

## Implementation Steps

1. **Update Configuration**: Replace `OtlpConfig` with `OtlpSdkConfig`
2. **Remove Dead Code**: Delete `new()`, `create_batch_metrics()`, `create_span_data()`, `convert_config()`
3. **Update Initialization**: Use SDK directly in `new_async()`, remove conversion layer
4. **Update Metrics Recording**: Use SDK methods in `record_batch_sent()`
5. **Update Trace Creation**: Use SDK methods in `start_send_batch_span()` and `ObservabilitySpan`
6. **Update Tests**: Update tests to use new config and verify SDK calls

## Benefits

- **Simplified Code**: Removed ~150 lines of dead code
- **SDK Consistency**: Uses standardized SDK patterns
- **Better Maintainability**: Less custom code to maintain
- **Proper OpenTelemetry**: SDK handles OpenTelemetry structure correctly

## Testing

All existing observability tests should be updated to:
- Use `OtlpSdkConfig` instead of `OtlpConfig`
- Verify SDK method calls instead of manual construction
- Test SDK-based behavior



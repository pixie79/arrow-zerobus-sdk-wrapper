# Rust API Contract: OTLP SDK Integration Update

**Feature**: 002-otlp-sdk-update  
**Date**: 2025-01-27

## API Changes

### Breaking Changes

All changes are breaking changes since no users currently depend on the wrapper.

### ObservabilityManager

#### Removed Methods

- `pub fn new(config: Option<OtlpConfig>) -> Option<Self>`
  - **Reason**: Dead code - always returned None, async initialization required
  - **Migration**: Use `new_async()` instead

- `fn create_batch_metrics(...) -> ResourceMetrics` (private)
  - **Reason**: Manual construction replaced by SDK methods
  - **Migration**: N/A (internal method)

- `fn create_span_data(...) -> SpanData` (private)
  - **Reason**: Manual construction replaced by SDK methods
  - **Migration**: N/A (internal method)

- `fn convert_config(config: OtlpConfig) -> OtlpLibraryConfig` (private)
  - **Reason**: Configuration conversion replaced by direct SDK config
  - **Migration**: N/A (internal method)

#### Modified Methods

- `pub async fn new_async(config: Option<OtlpSdkConfig>) -> Option<Self>`
  - **Change**: Parameter type changed from `OtlpConfig` to `OtlpSdkConfig`
  - **Behavior**: Uses SDK directly for initialization, no conversion layer
  - **Migration**: Update callers to use new `OtlpSdkConfig` structure

- `pub async fn record_batch_sent(&self, batch_size_bytes: usize, success: bool, latency_ms: u64)`
  - **Change**: Implementation now uses SDK methods instead of manual `ResourceMetrics` construction
  - **Behavior**: Metrics recorded via SDK, proper OpenTelemetry structure
  - **Migration**: No caller changes needed (same signature)

- `pub fn start_send_batch_span(&self, table_name: &str) -> ObservabilitySpan`
  - **Change**: Implementation now uses SDK for span creation
  - **Behavior**: Spans created via SDK with proper trace context
  - **Migration**: No caller changes needed (same signature)

#### Unchanged Methods

- `pub async fn flush(&self) -> Result<(), ZerobusError>`
- `pub async fn shutdown(&self) -> Result<(), ZerobusError>`

### Configuration Types

#### OtlpConfig → OtlpSdkConfig

**Old**:
```rust
pub struct OtlpConfig {
    pub endpoint: Option<String>,
    pub log_level: String,
    pub extra: HashMap<String, Value>,
}
```

**New**:
```rust
pub struct OtlpSdkConfig {
    pub endpoint: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub write_interval_secs: u64,
    pub log_level: String,
}
```

**Migration**:
- Remove `extra` field usage
- Add `output_dir` if file-based export needed
- Add `write_interval_secs` configuration
- `endpoint` and `log_level` remain similar

### ObservabilitySpan

#### Modified Implementation

- **Change**: Internal implementation uses SDK for span creation and export
- **Behavior**: Spans created via SDK, proper trace context management
- **Migration**: No caller changes needed (same public API)

## API Stability

### Public API Surface

The public API surface remains largely the same:
- `new_async()` - Still async initialization (parameter type changed)
- `record_batch_sent()` - Same signature
- `start_send_batch_span()` - Same signature
- `flush()` - Unchanged
- `shutdown()` - Unchanged

### Internal Changes

All breaking changes are internal:
- Removed dead code (synchronous `new()`)
- Replaced manual construction with SDK methods
- Simplified configuration structure

## Testing Impact

Tests will need updates to:
- Use `OtlpSdkConfig` instead of `OtlpConfig`
- Verify SDK method calls instead of manual construction
- Update expectations for SDK-based behavior


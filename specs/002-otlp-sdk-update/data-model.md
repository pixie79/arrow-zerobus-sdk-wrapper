# Data Model: OTLP SDK Integration Update

**Feature**: 002-otlp-sdk-update  
**Date**: 2025-01-27

## Entities

### OTLP SDK Configuration

**Purpose**: Configuration for the otlp-rust-service SDK, aligned with SDK requirements.

**Fields**:
- `endpoint: Option<String>` - OTLP endpoint URL for remote export (optional)
- `output_dir: Option<PathBuf>` - Output directory for file-based export (optional)
- `write_interval_secs: u64` - Write interval in seconds for file-based export (default: 5)
- `log_level: String` - Log level for tracing (default: "info")

**Relationships**:
- Used by `ObservabilityManager` for SDK initialization
- Replaces current `OtlpConfig` structure (breaking change allowed)

**Validation Rules**:
- If `endpoint` is provided, it should be a valid URL
- If `output_dir` is provided, it should be a valid directory path
- `write_interval_secs` must be > 0
- `log_level` must be a valid log level ("trace", "debug", "info", "warn", "error")

**State Transitions**: N/A (immutable configuration)

### Observability Manager

**Purpose**: Manages observability operations using the otlp-rust-service SDK.

**Fields**:
- `library: Option<Arc<OtlpLibrary>>` - SDK library instance (None if observability disabled or failed)

**Relationships**:
- Uses `OTLP SDK Configuration` for initialization
- Provides methods for metrics recording and trace generation
- Used by `ZerobusWrapper` for observability

**State Transitions**:
- **Uninitialized** → **Initialized**: After successful `new_async()` call
- **Initialized** → **Shutdown**: After `shutdown()` call
- **Initialized** → **Failed**: If SDK operations fail (library remains but operations may fail)

**Methods** (Public API):
- `new_async(config: Option<OtlpSdkConfig>) -> Option<Self>` - Async initialization
- `record_batch_sent(batch_size_bytes, success, latency_ms)` - Record metrics via SDK
- `start_send_batch_span(table_name) -> ObservabilitySpan` - Start trace span via SDK
- `flush() -> Result<(), ZerobusError>` - Flush pending data
- `shutdown() -> Result<(), ZerobusError>` - Shutdown SDK

**Removed Methods**:
- `new()` - Synchronous initialization (always returned None, dead code)
- `create_batch_metrics()` - Manual metrics construction (replaced by SDK)
- `create_span_data()` - Manual span construction (replaced by SDK)
- `convert_config()` - Configuration conversion (replaced by direct SDK config)

### Observability Span

**Purpose**: Span guard for batch transmission operations, using SDK for trace creation.

**Fields**:
- `table_name: String` - Target table name
- `start_time: SystemTime` - Span start time
- `sdk_span: Option<SdkSpan>` - SDK span instance (if observability enabled)

**Relationships**:
- Created by `ObservabilityManager::start_send_batch_span()`
- Automatically ends when dropped
- Uses SDK for span creation and export

**State Transitions**:
- **Active** → **Ended**: When dropped, span is ended and exported via SDK

**Changes**:
- Removed manual `SpanData` construction
- Uses SDK's span builder/recorder for proper trace context
- Automatic export handled by SDK

## Data Flow

### Metrics Recording Flow

1. `ZerobusWrapper` calls `ObservabilityManager::record_batch_sent()`
2. `ObservabilityManager` uses SDK method to record metrics
3. SDK handles metric structure creation and export
4. No manual `ResourceMetrics` construction

### Trace Generation Flow

1. `ZerobusWrapper` calls `ObservabilityManager::start_send_batch_span()`
2. `ObservabilityManager` uses SDK to create span
3. Returns `ObservabilitySpan` guard
4. When span is dropped, SDK handles span ending and export
5. No manual `SpanData` construction

## Configuration Migration

### Old Structure (OtlpConfig)
```rust
pub struct OtlpConfig {
    pub endpoint: Option<String>,
    pub log_level: String,
    pub extra: HashMap<String, Value>,
}
```

### New Structure (OtlpSdkConfig)
```rust
pub struct OtlpSdkConfig {
    pub endpoint: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub write_interval_secs: u64,
    pub log_level: String,
}
```

**Migration Notes**:
- Breaking change: Structure changed, `extra` field removed
- Direct mapping to SDK `Config` requirements
- Simpler, more explicit configuration


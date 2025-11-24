# Data Model: Zerobus SDK Wrapper

**Feature**: 001-zerobus-wrapper  
**Date**: 2025-11-23

## Entities

### WrapperConfiguration

Represents the complete configuration for initializing the wrapper.

**Attributes**:
- `zerobus_endpoint: String` - Zerobus endpoint URL (required)
- `unity_catalog_url: Option<String>` - Unity Catalog URL for authentication (required for SDK)
- `client_id: Option<String>` - OAuth2 client ID (required for SDK)
- `client_secret: Option<String>` - OAuth2 client secret (required for SDK)
- `table_name: String` - Target table name in Zerobus (required)
- `observability_enabled: bool` - Enable/disable OpenTelemetry observability (default: false)
- `observability_config: Option<OtlpConfig>` - OpenTelemetry configuration (optional)
- `debug_enabled: bool` - Enable/disable debug file output (default: false)
- `debug_output_dir: Option<PathBuf>` - Output directory for debug files (required if debug_enabled)
- `debug_flush_interval_secs: u64` - Debug file flush interval in seconds (default: 5)
- `debug_max_file_size: Option<u64>` - Maximum debug file size in bytes before rotation (optional)
- `retry_max_attempts: u32` - Maximum retry attempts for transient failures (default: 5)
- `retry_base_delay_ms: u64` - Base delay in milliseconds for exponential backoff (default: 100)
- `retry_max_delay_ms: u64` - Maximum delay in milliseconds for exponential backoff (default: 30000)

**Validation Rules**:
- `zerobus_endpoint` must be a valid URL starting with `https://` or `http://`
- If `debug_enabled` is true, `debug_output_dir` must be provided
- `retry_max_attempts` must be > 0
- `debug_flush_interval_secs` must be > 0

**State Transitions**: None (immutable configuration)

### ZerobusWrapper

Main wrapper instance for sending data to Zerobus.

**Attributes**:
- `config: Arc<WrapperConfiguration>` - Configuration (immutable)
- `sdk: Arc<Mutex<ZerobusSdk>>` - Zerobus SDK instance (thread-safe)
- `stream: Arc<Mutex<Option<ZerobusStream>>>` - Active stream (lazy initialization)
- `observability: Option<Arc<OtlpLibrary>>` - OpenTelemetry library instance (optional)
- `debug_writer: Option<Arc<DebugWriter>>` - Debug file writer (optional)
- `retry_config: RetryConfig` - Retry configuration

**State Transitions**:
- **Uninitialized** → **Initialized**: After successful `new()` or `new_with_config()`
- **Initialized** → **Stream Active**: After first successful data transmission (lazy stream creation)
- **Stream Active** → **Stream Error**: On stream failure (will retry on next transmission)
- **Initialized** → **Shutdown**: After `shutdown()` call

**Thread Safety**: All operations are thread-safe via internal synchronization (Arc<Mutex>)

### DataBatch

Represents a collection of structured data records in Arrow RecordBatch format.

**Attributes**:
- `record_batch: RecordBatch` - Arrow RecordBatch containing schema and data
- `schema: SchemaRef` - Arrow schema reference
- `num_rows: usize` - Number of rows in the batch
- `size_bytes: usize` - Approximate size in bytes

**Validation Rules**:
- `record_batch` must be valid Arrow RecordBatch
- `num_rows` must match actual row count in record_batch
- No size limit (wrapper accepts batches of any size)

**State Transitions**: None (immutable data structure)

### TransmissionResult

Represents the outcome of a data transmission operation.

**Attributes**:
- `success: bool` - Whether transmission succeeded
- `error: Option<ZerobusError>` - Error information if transmission failed
- `attempts: u32` - Number of retry attempts made
- `latency_ms: Option<u64>` - Transmission latency in milliseconds (if successful)
- `batch_size_bytes: usize` - Size of transmitted batch in bytes

**Validation Rules**:
- If `success` is true, `error` must be None
- If `success` is false, `error` must be Some
- `attempts` must be >= 1

**State Transitions**: None (result object, created after operation completes)

### RetryConfig

Configuration for retry behavior.

**Attributes**:
- `max_attempts: u32` - Maximum number of retry attempts (default: 5)
- `base_delay_ms: u64` - Base delay in milliseconds for exponential backoff (default: 100)
- `max_delay_ms: u64` - Maximum delay in milliseconds (default: 30000)
- `jitter: bool` - Enable jitter in backoff calculation (default: true)

**Validation Rules**:
- `max_attempts` must be > 0
- `base_delay_ms` must be > 0
- `max_delay_ms` must be >= `base_delay_ms`

**State Transitions**: None (immutable configuration)

### DebugWriter

Handles debug file output for Arrow and Protobuf formats.

**Attributes**:
- `output_dir: PathBuf` - Output directory for debug files
- `arrow_writer: Option<ArrowFileWriter>` - Arrow IPC file writer
- `protobuf_writer: Option<ProtobufFileWriter>` - Protobuf file writer
- `flush_interval: Duration` - Interval for periodic flushing
- `max_file_size: Option<u64>` - Maximum file size before rotation
- `last_flush: Instant` - Timestamp of last flush operation

**State Transitions**:
- **Disabled** → **Enabled**: When debug output is enabled in configuration
- **Enabled** → **Writing**: When first batch is written
- **Writing** → **Rotating**: When file size exceeds max_file_size
- **Writing** → **Flushing**: When flush_interval elapses

### ZerobusError

Error type for wrapper operations.

**Variants**:
- `ConfigurationError(String)` - Invalid configuration
- `AuthenticationError(String)` - Authentication failure
- `ConnectionError(String)` - Network/connection error
- `ConversionError(String)` - Arrow to Protobuf conversion failure
- `TransmissionError(String)` - Data transmission failure
- `RetryExhausted(String)` - All retry attempts exhausted
- `TokenRefreshError(String)` - Token refresh failure

**Validation Rules**: All error variants must contain descriptive error messages

## Relationships

- **WrapperConfiguration** → **ZerobusWrapper**: One-to-one (wrapper created with configuration)
- **ZerobusWrapper** → **DataBatch**: One-to-many (wrapper can send multiple batches)
- **ZerobusWrapper** → **TransmissionResult**: One-to-many (each batch transmission produces a result)
- **ZerobusWrapper** → **DebugWriter**: One-to-one (optional, created if debug enabled)
- **ZerobusWrapper** → **RetryConfig**: One-to-one (retry config from wrapper config)
- **DataBatch** → **TransmissionResult**: One-to-one (each batch produces one result)

## Identity & Uniqueness

- **WrapperConfiguration**: Identified by combination of endpoint, table_name, and configuration hash
- **ZerobusWrapper**: Single instance per configuration (can be cloned via Arc for sharing)
- **DataBatch**: Identified by content hash (for deduplication if needed)
- **TransmissionResult**: Identified by operation ID (timestamp + batch hash)

## Lifecycle

1. **Configuration Creation**: User creates WrapperConfiguration with required fields
2. **Wrapper Initialization**: ZerobusWrapper created from configuration, SDK initialized
3. **Stream Creation**: Stream created lazily on first data transmission
4. **Data Transmission**: DataBatch sent, converted to Protobuf, transmitted via SDK
5. **Result Generation**: TransmissionResult created with success/failure status
6. **Retry Logic**: If transient failure, retry with exponential backoff + jitter
7. **Token Refresh**: If authentication error, refresh token and retry
8. **Shutdown**: Wrapper shutdown gracefully closes stream and cleans up resources

## Data Volume Assumptions

- Batch sizes: No upper limit (wrapper accepts any size)
- Typical batch size: 1MB - 10MB for performance measurement
- Debug files: Rotated when max_file_size reached (configurable, default: 100MB)
- Concurrent operations: Multiple threads/async tasks can send batches simultaneously
- Memory usage: Bounded by batch size + internal buffers (typically < 50MB per wrapper instance)


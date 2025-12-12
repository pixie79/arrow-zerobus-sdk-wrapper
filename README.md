# Arrow Zerobus SDK Wrapper

Cross-platform Rust SDK wrapper for Databricks Zerobus with Python bindings. Provides a unified API for sending Arrow RecordBatch data to Zerobus with automatic protocol conversion, authentication, retry logic, and observability.

## Features

- **Rust SDK**: Native Rust API for sending Arrow RecordBatch data to Zerobus
- **Python Bindings**: Python 3.11+ support via PyO3 with zero-copy data transfer
- **Automatic Retry**: Exponential backoff with jitter for transient failures
- **Token Refresh**: Automatic authentication token refresh for long-running operations
- **Observability**: OpenTelemetry metrics and traces integration
- **Debug Output**: Optional Arrow and Protobuf file output for debugging with independent control per format
- **File Retention**: Automatic cleanup of old rotated debug files to prevent disk space issues
- **Writer Disabled Mode**: Disable Zerobus SDK transmission while maintaining debug file output for local development and testing
- **Per-Row Error Tracking**: Identify which specific rows failed, enabling partial batch success and efficient quarantine workflows
- **Error Analysis**: Group errors by type, track statistics, and analyze patterns for debugging
- **Zerobus Limits Compliance**: Automatic validation and enforcement of Zerobus service limits (2000 columns, 4MB records, ASCII-only names, correct type mappings)
- **Thread-Safe**: Concurrent operations from multiple threads/async tasks
- **Cross-Platform**: Linux, macOS, Windows support
- **Cross-Platform**: Linux, macOS, Windows support

## Requirements

- Rust 1.75+ (edition 2021)
- Python 3.11+ (for Python bindings)
- Databricks workspace with Zerobus enabled
- OAuth2 credentials (client_id, client_secret)
- Unity Catalog URL

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
arrow-zerobus-sdk-wrapper = { version = "0.1.0", path = "../arrow-zerobus-sdk-wrapper" }
arrow = "57"
tokio = { version = "1.35", features = ["full"] }
```

### Python

```bash
pip install arrow-zerobus-sdk-wrapper
```

Or from source:

```bash
pip install -e /path/to/arrow-zerobus-sdk-wrapper
```

## Quick Start

### Rust

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use arrow::record_batch::RecordBatch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WrapperConfiguration::new(
        "https://your-workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    )
    .with_credentials("client_id".to_string(), "client_secret".to_string())
    .with_unity_catalog("https://unity-catalog-url".to_string());
    
    let wrapper = ZerobusWrapper::new(config).await?;
    let batch = create_record_batch()?;
    let result = wrapper.send_batch(batch).await?;
    
    if result.success {
        println!("Batch sent successfully!");
        
        // Handle per-row errors (if any)
        if result.is_partial_success() {
            println!("⚠️  Partial success: {} succeeded, {} failed", 
                     result.successful_count, result.failed_count);
            
            // Extract and quarantine failed rows
            if let Some(failed_batch) = result.extract_failed_batch(&batch) {
                // Quarantine failed_batch
                println!("Quarantining {} failed rows", failed_batch.num_rows());
            }
            
            // Extract and write successful rows
            if let Some(successful_batch) = result.extract_successful_batch(&batch) {
                // Write successful_batch to main table
                println!("Writing {} successful rows", successful_batch.num_rows());
            }
        }
        
        // Analyze error patterns
        if result.has_failed_rows() {
            let stats = result.get_error_statistics();
            println!("Success rate: {:.1}%", stats.success_rate * 100.0);
            
            let grouped = result.group_errors_by_type();
            for (error_type, indices) in &grouped {
                println!("  {}: {} rows", error_type, indices.len());
            }
        }
    }
    
    wrapper.shutdown().await?;
    Ok(())
}
```

### Python

See [examples/python_example.py](examples/python_example.py) for a complete example.

```python
import asyncio
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def main():
    wrapper = ZerobusWrapper(
        endpoint="https://your-workspace.cloud.databricks.com",
        table_name="my_table",
        client_id="client_id",
        client_secret="client_secret",
        unity_catalog_url="https://unity-catalog-url",
    )
    
    # Create Arrow RecordBatch
    schema = pa.schema([
        pa.field("id", pa.int64()),
        pa.field("name", pa.string()),
    ])
    arrays = [
        pa.array([1, 2, 3], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
    ]
    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
    
    # Send batch
    result = await wrapper.send_batch(batch)
    
    if result.success:
        print("Batch sent successfully!")
        
        # Handle per-row errors (if any)
        if result.is_partial_success():
            print(f"⚠️  Partial success: {result.successful_count} succeeded, {result.failed_count} failed")
            
            # Extract and quarantine failed rows
            failed_batch = result.extract_failed_batch(batch)
            if failed_batch is not None:
                print(f"Quarantining {failed_batch.num_rows} failed rows")
            
            # Extract and write successful rows
            successful_batch = result.extract_successful_batch(batch)
            if successful_batch is not None:
                print(f"Writing {successful_batch.num_rows} successful rows")
        
        # Analyze error patterns
        if result.has_failed_rows():
            stats = result.get_error_statistics()
            print(f"Success rate: {stats['success_rate'] * 100:.1}%")
            
            grouped = result.group_errors_by_type()
            for error_type, indices in grouped.items():
                print(f"  {error_type}: {len(indices)} rows")
    
    await wrapper.shutdown()

asyncio.run(main())
```

## Writer Disabled Mode

The wrapper supports a "writer disabled" mode that allows you to test data conversion logic and write debug files without making network calls to Zerobus. This is useful for:

- **Local Development**: Test data transformations without Databricks workspace access
- **CI/CD Testing**: Validate data format without requiring credentials
- **Performance Testing**: Benchmark conversion logic without network overhead

### Usage

**Rust:**
```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use std::path::PathBuf;

let config = WrapperConfiguration::new(
    "https://workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_debug_output(PathBuf::from("./debug_output"))
.with_zerobus_writer_disabled(true);  // Enable disabled mode

let wrapper = ZerobusWrapper::new(config).await?;
// No credentials required when writer is disabled
let result = wrapper.send_batch(batch).await?;
// Debug files written, no network calls made
```

**Python:**
```python
wrapper = ZerobusWrapper(
    endpoint="https://workspace.cloud.databricks.com",
    table_name="my_table",
    debug_enabled=True,
    debug_output_dir="./debug_output",
    zerobus_writer_disabled=True,  # Enable disabled mode
    # No credentials required when writer is disabled
)

result = await wrapper.send_batch(batch)
# Debug files written, no network calls made
```

**Note**: When `zerobus_writer_disabled` is `true`, at least one debug format must be enabled. Credentials are optional when writer is disabled.

## Debug Output Configuration

The wrapper supports flexible debug output configuration with independent control over Arrow and Protobuf file generation, automatic file retention, and improved file rotation.

### Separate Arrow/Protobuf Flags

You can enable Arrow and Protobuf debug output independently:

**Rust:**
```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use std::path::PathBuf;

// Enable only Arrow debug files
let config = WrapperConfiguration::new(
    "https://workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_debug_arrow_enabled(true)      // Enable Arrow files (.arrows)
.with_debug_protobuf_enabled(false)  // Disable Protobuf files
.with_debug_output(PathBuf::from("./debug_output"));

// Enable only Protobuf debug files
let config = WrapperConfiguration::new(...)
.with_debug_arrow_enabled(false)
.with_debug_protobuf_enabled(true)   // Enable Protobuf files (.proto)
.with_debug_output(PathBuf::from("./debug_output"));

// Enable both formats
let config = WrapperConfiguration::new(...)
.with_debug_arrow_enabled(true)
.with_debug_protobuf_enabled(true)
.with_debug_output(PathBuf::from("./debug_output"));
```

**Python:**
```python
# Enable only Arrow debug files
config = PyWrapperConfiguration(
    endpoint="https://workspace.cloud.databricks.com",
    table_name="my_table",
    debug_arrow_enabled=True,      # Enable Arrow files
    debug_protobuf_enabled=False,  # Disable Protobuf files
    debug_output_dir="./debug_output"
)

# Enable only Protobuf debug files
config = PyWrapperConfiguration(
    endpoint="https://workspace.cloud.databricks.com",
    table_name="my_table",
    debug_arrow_enabled=False,
    debug_protobuf_enabled=True,   # Enable Protobuf files
    debug_output_dir="./debug_output"
)
```

### File Retention

Control how many rotated debug files are retained to manage disk space:

**Rust:**
```rust
// Keep last 20 rotated files per type (default: 10)
let config = WrapperConfiguration::new(...)
.with_debug_arrow_enabled(true)
.with_debug_output(PathBuf::from("./debug_output"))
.with_debug_max_files_retained(Some(20));

// Unlimited retention (no automatic cleanup)
let config = WrapperConfiguration::new(...)
.with_debug_arrow_enabled(true)
.with_debug_output(PathBuf::from("./debug_output"))
.with_debug_max_files_retained(None);
```

**Python:**
```python
# Keep last 20 rotated files per type
config = PyWrapperConfiguration(
    endpoint="https://workspace.cloud.databricks.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_output_dir="./debug_output",
    debug_max_files_retained=20  # Default: 10, None = unlimited
)

# Unlimited retention
config = PyWrapperConfiguration(
    endpoint="https://workspace.cloud.databricks.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_output_dir="./debug_output",
    debug_max_files_retained=None  # No automatic cleanup
)
```

### Configuration via YAML

```yaml
debug:
  arrow_enabled: true          # Enable Arrow debug files
  protobuf_enabled: false      # Disable Protobuf debug files
  output_dir: "/tmp/debug"
  max_files_retained: 20       # Keep last 20 rotated files (default: 10)
  flush_interval_secs: 5
  max_file_size: 10485760
```

### Configuration via Environment Variables

```bash
export DEBUG_ARROW_ENABLED=true
export DEBUG_PROTOBUF_ENABLED=false
export DEBUG_OUTPUT_DIR=/tmp/debug
export DEBUG_MAX_FILES_RETAINED=20
export DEBUG_FLUSH_INTERVAL_SECS=5
```

### Backward Compatibility

The legacy `debug_enabled` flag still works. When set to `true` and new flags are not explicitly set, both Arrow and Protobuf formats are enabled:

```rust
// Legacy code - still works, enables both formats
let config = WrapperConfiguration::new(...)
.with_debug_output(PathBuf::from("./debug_output"));
// Note: debug_enabled must be set to true explicitly; with_debug_output() does not enable debugging by itself
```

```python
# Legacy Python code - still works
config = PyWrapperConfiguration(
    endpoint="https://workspace.cloud.databricks.com",
    table_name="my_table",
    debug_enabled=True,  # Enables both Arrow and Protobuf when new flags not set
    debug_output_dir="./debug_output"
)
```

### File Rotation Improvements

File rotation has been improved to prevent recursive timestamp appending and filename length errors:

- **Timestamp Extraction**: Base filename is extracted before appending new timestamp, preventing filenames like `file_20250101_120000_20250101_120001`
- **Sequential Fallback**: When filenames would exceed filesystem limits, sequential numbering (`_1`, `_2`, etc.) is used instead of timestamps
- **Automatic Cleanup**: Old rotated files are automatically deleted when retention limit is exceeded

## Zerobus Limits Compliance

The wrapper automatically validates and enforces Zerobus service limits to prevent API errors and ensure compatibility:

### Column Count Limit

- **Maximum**: 2000 columns per table (Zerobus limit)
- **Validation**: Descriptors with more than 2000 fields are rejected during schema generation
- **Error**: Clear error message indicates the limit and current field count

### Record Size Limit

- **Maximum**: 4MB per message (4,194,285 bytes payload + 19 bytes headers)
- **Validation**: Records exceeding the limit are rejected before transmission
- **Error**: Clear error message indicates the limit and actual record size

```rust
// Records exceeding 4MB will be rejected with a clear error
let result = wrapper.send_batch(large_batch).await?;
if !result.success {
    // Check for size limit errors in failed_rows
    for (idx, error) in &result.failed_rows {
        if let ZerobusError::ConversionError(msg) = error {
            if msg.contains("exceeds Zerobus limit") {
                println!("Row {} exceeds 4MB limit", idx);
            }
        }
    }
}
```

### Name Validation

- **Table Names**: Must contain only ASCII letters, digits, and underscores
- **Column Names**: Must contain only ASCII letters, digits, and underscores
- **Validation**: Names are validated during configuration and schema generation
- **Error**: Clear error message indicates the invalid name and requirement

```rust
// Invalid table name will be rejected
let config = WrapperConfiguration::new(
    "https://workspace.cloud.databricks.com".to_string(),
    "table-name".to_string(),  // ❌ Invalid: contains hyphen
);
let result = config.validate();
// Returns ConfigurationError: "table_name must contain only ASCII letters, digits, and underscores"
```

### Type Mapping Compliance

The wrapper ensures correct type mappings per Zerobus specification:

- **Date32** → `Int32` (days since epoch) ✅
- **Date64** → `Int64` (milliseconds since epoch)
- **Timestamp** → `Int64` (microseconds since epoch) ✅
- **Integer types** → `Int32` or `Int64` as appropriate ✅
- **String** → `String` ✅
- **Binary** → `Bytes` ✅
- **Arrays** → `repeated TYPE` ✅
- **Structs** → `message Nested { FIELDS }` ✅

All type mappings are validated to ensure compatibility with Zerobus requirements.

## Building

### Rust

```bash
cargo build --release
```

### Python Bindings

```bash
pip install maturin
maturin build --release
pip install target/wheels/*.whl
```

## Testing

### Rust

**Note**: When running tests with the `python` feature enabled, PyO3 needs to link against the Python library. You can either:

1. **Use the helper script** (recommended):
   ```bash
   ./scripts/test.sh --all-features
   ```

2. **Set PYO3_PYTHON manually**:
   ```bash
   # Find your Python executable
   which python3.11 || which python3
   
   # Run tests with Python feature
   PYO3_PYTHON=/path/to/python3 cargo test --all-features
   ```

The helper script automatically detects and uses Python 3.11+ from your PATH.

### Rust Tests

```bash
# Run all tests (without Python feature)
cargo test

# Run all tests including Python bindings
./scripts/test.sh --all-features

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Xml --output-dir coverage
```

### Python Tests

**Note**: Python tests require the Python extension to be built first. The tests use `pytest-forked` to work around PyO3 GIL issues that can cause pytest to hang after tests.

```bash
# Recommended: Use the helper script (handles setup automatically)
./scripts/test-python.sh

# Manual setup:
# 1. Build Python extension
maturin develop --release

# 2. Install test dependencies
pip install pytest pytest-cov pytest-forked

# 3. Run tests with PyO3 workaround
export PYO3_NO_PYTHON_VERSION_CHECK=1
pytest tests/python/ -v --forked

# Run with coverage
pytest --cov=arrow_zerobus_sdk_wrapper tests/python/ --forked
```

**PyO3 Pytest Workaround**: The `--forked` flag ensures each test runs in a separate process, preventing GIL (Global Interpreter Lock) deadlocks that can cause pytest to hang. The `conftest.py` file includes additional fixtures to ensure proper Python initialization and cleanup.

### Performance Benchmarks

```bash
# Run latency benchmarks
cargo bench --bench latency

# Run throughput benchmarks
cargo bench --bench throughput
```

## Performance

- **Latency**: p95 latency under 150ms for batches up to 10MB
- **Success Rate**: 99.999% under normal network conditions
- **Concurrency**: Thread-safe, supports concurrent operations

## Debug File Inspection with DuckDB

When debug output is enabled, the wrapper writes Arrow and Protobuf files to disk for inspection. You can use DuckDB to read and analyze these files using the [Arrow IPC support in DuckDB](https://duckdb.org/2025/05/23/arrow-ipc-support-in-duckdb).

### Installing the Arrow Extension

First, install and load the DuckDB Arrow community extension:

```sql
INSTALL arrow FROM community;
LOAD arrow;
```

### Reading Arrow IPC Files

Arrow files are written in Arrow IPC stream format (`.arrow` or `.arrows` extension) and can be read directly by DuckDB:

```sql
-- Read Arrow IPC file using read_arrow() function
SELECT * FROM read_arrow('debug_output/zerobus/arrow/table.arrow');

-- DuckDB supports replacement scans - you can omit read_arrow() 
-- if the filename ends with .arrow or .arrows
SELECT * FROM 'debug_output/zerobus/arrow/table.arrow';

-- Read multiple Arrow files (including rotated files)
SELECT * FROM read_arrow('debug_output/zerobus/arrow/*.arrow');

-- Query specific columns
SELECT id, name, score 
FROM 'debug_output/zerobus/arrow/table.arrow'
WHERE score > 90;

-- Aggregate data
SELECT 
    COUNT(*) as total_rows,
    AVG(score) as avg_score,
    MAX(score) as max_score
FROM 'debug_output/zerobus/arrow/table.arrow';
```

### Reading from HTTP

You can also read Arrow IPC files over HTTP using DuckDB's `httpfs` extension:

```sql
INSTALL httpfs;
LOAD httpfs;
LOAD arrow;

-- Read Arrow IPC file from HTTP server
SELECT * FROM read_arrow('http://localhost:8008/table.arrow');
```

### Reading from stdin

You can pipe Arrow IPC data directly to DuckDB:

```bash
# Using curl to fetch and pipe to DuckDB
URL="http://localhost:8008/table.arrow"
SQL="LOAD arrow; FROM read_arrow('/dev/stdin') SELECT count(*);"
curl -s "$URL" | duckdb -c "$SQL"
```

### Example: Analyzing Debug Files

```sql
-- Load the Arrow extension
LOAD arrow;

-- Read all Arrow files from debug output
SELECT 
    COUNT(*) as total_rows,
    COUNT(DISTINCT file_name) as num_files
FROM read_arrow('debug_output/zerobus/arrow/*.arrow');

-- Analyze specific data
SELECT 
    id,
    name,
    score,
    COUNT(*) OVER () as total_count
FROM 'debug_output/zerobus/arrow/table.arrow'
WHERE score > 90
ORDER BY score DESC;
```

### Python Example with DuckDB

```python
import duckdb

# Connect to DuckDB
conn = duckdb.connect()

# Install and load Arrow extension
conn.execute("INSTALL arrow FROM community")
conn.execute("LOAD arrow")

# Read Arrow debug files (using replacement scan)
result = conn.execute("""
    SELECT * 
    FROM 'debug_output/zerobus/arrow/table.arrow'
    LIMIT 100
""").fetchdf()

print(result)

# Analyze data across multiple files
stats = conn.execute("""
    SELECT 
        COUNT(*) as total_rows,
        COUNT(DISTINCT file_name) as num_files
    FROM read_arrow('debug_output/zerobus/arrow/*.arrow')
""").fetchdf()

print(stats)
```

### Reading Protobuf Files

Protobuf files contain binary Protobuf messages. To read them with DuckDB, you'll need to:

1. **Convert Protobuf to Arrow IPC first** (using a Protobuf parser), or
2. **Use Python with PyArrow** to convert Protobuf to Arrow format

For Protobuf files, you can use Python to convert to Arrow IPC format, then read with DuckDB:

```python
import duckdb
import pyarrow as pa
from google.protobuf.message import Message

# Read Protobuf file and convert to Arrow IPC
# (This requires knowledge of your Protobuf schema)
def protobuf_to_arrow_ipc(proto_file, schema):
    # Parse Protobuf messages and convert to Arrow
    # Then write as Arrow IPC format
    # Implementation depends on your specific Protobuf schema
    pass

# Convert Protobuf to Arrow IPC file
protobuf_to_arrow_ipc('debug_output/zerobus/proto/table.proto', schema)

# Then use DuckDB to read the converted Arrow IPC file
conn = duckdb.connect()
conn.execute("INSTALL arrow FROM community")
conn.execute("LOAD arrow")
result = conn.execute("SELECT * FROM 'converted_table.arrow'").fetchdf()
```

Alternatively, if you have the Protobuf schema definition, you can use tools like `protoc` to generate code for parsing, then convert to Arrow IPC format that DuckDB can read.

### Notes

- **Arrow IPC Format**: Files use Arrow IPC stream format (recommended extension: `.arrows`)
- **Extension Support**: DuckDB can read files with `.arrow` or `.arrows` extensions directly (replacement scans)
- **Multi-file Reading**: Supports reading multiple files using glob patterns (`*.arrow`)
- **Performance**: Arrow IPC format is optimized for fast encoding/decoding and zero-copy data transfer
- **Protobuf Files**: Require conversion to Arrow IPC format first before reading with DuckDB
- **File Rotation**: Rotated files (with timestamp suffixes like `table_20251212_143022.arrows`) can be read using glob patterns
- **File Retention**: Old rotated files are automatically cleaned up based on `debug_max_files_retained` setting (default: 10 files per type)
- **Flush Before Reading**: Debug files are written incrementally, so you may need to call `wrapper.flush()` before reading

For more details, see the [official DuckDB Arrow IPC support documentation](https://duckdb.org/2025/05/23/arrow-ipc-support-in-duckdb).

## Development

### Pre-commit Hooks

The repository includes a pre-commit hook to ensure version consistency across all configuration files. Before each commit, the hook verifies that version numbers match in:

- `Cargo.toml`
- `pyproject.toml`
- `CHANGELOG.md` (latest release)

To install the pre-commit hook:

```bash
./scripts/install_pre_commit_hook.sh
```

To manually check version consistency:

```bash
./scripts/check_version.sh
```

### Version Management

When releasing a new version, ensure all version numbers are updated:

1. Update `Cargo.toml`: `version = "X.Y.Z"`
2. Update `pyproject.toml`: `version = "X.Y.Z"`
3. Update `CHANGELOG.md`: Add new release section `## [X.Y.Z] - YYYY-MM-DD`

The pre-commit hook and CI pipeline will verify version consistency automatically.

For more details, see [Version Management Guide](docs/VERSION_MANAGEMENT.md).

## Documentation

- [API Documentation](docs/api.md)
- [Quickstart Guide](specs/001-zerobus-wrapper/quickstart.md)
- [Architecture](docs/architecture.md)
- [Version Management Guide](docs/VERSION_MANAGEMENT.md)

## License

MIT OR Apache-2.0

# Arrow Zerobus SDK Wrapper

Cross-platform Rust SDK wrapper for Databricks Zerobus with Python bindings. Provides a unified API for sending Arrow RecordBatch data to Zerobus with automatic protocol conversion, authentication, retry logic, and observability.

## Features

- **Rust SDK**: Native Rust API for sending Arrow RecordBatch data to Zerobus
- **Python Bindings**: Python 3.11+ support via PyO3 with zero-copy data transfer
- **Automatic Retry**: Exponential backoff with jitter for transient failures
- **Token Refresh**: Automatic authentication token refresh for long-running operations
- **Observability**: OpenTelemetry metrics and traces integration
- **Debug Output**: Optional Arrow and Protobuf file output for debugging
- **Thread-Safe**: Concurrent operations from multiple threads/async tasks
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
    
    result = await wrapper.send_batch(batch)
    
    if result.success:
        print(f"Batch sent successfully in {result.latency_ms}ms")
    
    await wrapper.shutdown()

asyncio.run(main())
```

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

**Note**: When running tests with the `python` feature enabled, you may need to set the `PYO3_PYTHON` environment variable to point to your Python executable:

```bash
# Find your Python executable
which python3.11 || which python3

# Run tests with Python feature
PYO3_PYTHON=/path/to/python3 cargo test --all-features
```

This is required because PyO3 needs to link against the Python library when building Python bindings. Tests

```bash
# Run all tests
cargo test

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Xml --output-dir coverage
```

### Python Tests

```bash
# Run Python tests
pytest tests/python/

# Run with coverage
pytest --cov=arrow_zerobus_sdk_wrapper tests/python/
```

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
- **File Rotation**: Rotated files (with timestamp suffixes) can be read using glob patterns
- **Flush Before Reading**: Debug files are written incrementally, so you may need to call `wrapper.flush()` before reading

For more details, see the [official DuckDB Arrow IPC support documentation](https://duckdb.org/2025/05/23/arrow-ipc-support-in-duckdb).

## Documentation

- [API Documentation](docs/api.md)
- [Quickstart Guide](specs/001-zerobus-wrapper/quickstart.md)
- [Architecture](docs/architecture.md)

## License

MIT OR Apache-2.0

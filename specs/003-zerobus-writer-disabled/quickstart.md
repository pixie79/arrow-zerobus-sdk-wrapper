# Quickstart: Zerobus Writer Disabled Mode

**Feature**: 003-zerobus-writer-disabled  
**Date**: 2025-12-11

## Overview

The Zerobus Writer Disabled Mode allows you to test data conversion logic and write debug files without making network calls to Zerobus. This is useful for:

- **Local Development**: Test data transformations without Databricks workspace access
- **CI/CD Testing**: Validate data format without requiring credentials
- **Performance Testing**: Benchmark conversion logic without network overhead

## Prerequisites

- Rust 1.75+ or Python 3.11+
- Arrow RecordBatch data to test
- Debug output directory (for inspecting files)

## Quick Start

### Rust

#### Basic Usage

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use std::path::PathBuf;
use arrow::record_batch::RecordBatch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration with writer disabled mode
    let config = WrapperConfiguration::new(
        "https://workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    )
    .with_debug_output(PathBuf::from("./debug_output"))
    .with_zerobus_writer_disabled(true);  // Enable disabled mode
    
    // Initialize wrapper (no credentials needed when disabled)
    let wrapper = ZerobusWrapper::new(config).await?;
    
    // Create test batch
    let batch = create_test_batch()?;
    
    // Send batch - writes debug files but skips network calls
    let result = wrapper.send_batch(batch).await?;
    
    if result.success {
        println!("✅ Conversion succeeded! Check debug files in ./debug_output");
    }
    
    wrapper.shutdown().await?;
    Ok(())
}
```

#### Configuration Validation

```rust
use arrow_zerobus_sdk_wrapper::WrapperConfiguration;

// This will fail validation - debug must be enabled when writer is disabled
let config = WrapperConfiguration::new(
    "https://workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_zerobus_writer_disabled(true);  // But debug_enabled is false (default)

match config.validate() {
    Err(e) => println!("Validation error: {}", e),
    Ok(_) => println!("Configuration valid"),
}
```

### Python

#### Basic Usage

```python
import asyncio
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def main():
    # Create wrapper with writer disabled mode
    wrapper = ZerobusWrapper(
        endpoint="https://workspace.cloud.databricks.com",
        table_name="my_table",
        debug_enabled=True,
        debug_output_dir="./debug_output",
        zerobus_writer_disabled=True,  # Enable disabled mode
        # Note: credentials not required when writer is disabled
    )
    
    # Create test batch
    schema = pa.schema([
        pa.field("id", pa.int64()),
        pa.field("name", pa.string()),
    ])
    arrays = [
        pa.array([1, 2, 3], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
    ]
    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
    
    # Send batch - writes debug files but skips network calls
    result = await wrapper.send_batch(batch)
    
    if result.success:
        print("✅ Conversion succeeded! Check debug files in ./debug_output")
    
    await wrapper.shutdown()

asyncio.run(main())
```

#### CI/CD Testing Example

```python
import os
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def test_data_conversion():
    """Test data conversion in CI/CD without credentials."""
    wrapper = ZerobusWrapper(
        endpoint=os.getenv("ZEROBUS_ENDPOINT", "https://test.cloud.databricks.com"),
        table_name="test_table",
        debug_enabled=True,
        debug_output_dir=os.getenv("DEBUG_OUTPUT_DIR", "./test_debug"),
        zerobus_writer_disabled=True,  # No credentials needed
    )
    
    # Test batch
    batch = create_test_batch()
    result = await wrapper.send_batch(batch)
    
    # Verify conversion succeeded
    assert result.success
    
    # Verify debug files exist
    assert os.path.exists("./test_debug/zerobus/arrow/test_table.arrow")
    assert os.path.exists("./test_debug/zerobus/proto/test_table.proto")
    
    await wrapper.shutdown()
```

## What Happens When Writer is Disabled

When `zerobus_writer_disabled` is `true`:

1. ✅ **Arrow debug files are written** - You can inspect the input data
2. ✅ **Protobuf debug files are written** - You can see the converted format
3. ✅ **Protobuf descriptors are written** - You can inspect the schema
4. ✅ **Arrow-to-Protobuf conversion executes** - Conversion logic is tested
5. ❌ **SDK initialization is skipped** - No Zerobus SDK calls
6. ❌ **Stream creation is skipped** - No network connections
7. ❌ **Data transmission is skipped** - No data sent to Zerobus

## Inspecting Debug Files

After running with writer disabled mode, you can inspect the generated files:

### Arrow Files

```bash
# Arrow files are in Arrow IPC format
ls -lh debug_output/zerobus/arrow/my_table.arrow

# You can read them with DuckDB or PyArrow
```

### Protobuf Files

```bash
# Protobuf files contain binary messages
ls -lh debug_output/zerobus/proto/my_table.proto

# Descriptors are in the descriptors directory
ls -lh debug_output/zerobus/descriptors/my_table.pb
```

### Using DuckDB to Inspect Arrow Files

```sql
-- Install and load Arrow extension
INSTALL arrow FROM community;
LOAD arrow;

-- Read Arrow file
SELECT * FROM 'debug_output/zerobus/arrow/my_table.arrow';
```

## Configuration Options

### Required When Writer Disabled

- `debug_enabled: true` - Debug output must be enabled
- `debug_output_dir` - Output directory for debug files

### Optional When Writer Disabled

- `client_id` - Not required (no authentication needed)
- `client_secret` - Not required (no authentication needed)
- `unity_catalog_url` - Not required (no SDK initialization)

### Still Required

- `endpoint` - Still required (used for configuration, not network calls)
- `table_name` - Still required (used for file naming)

## Performance

When writer disabled mode is enabled:

- **Operation time**: < 50ms (excluding file I/O)
- **Network calls**: Zero
- **Memory usage**: Same as normal operation (conversion still happens)
- **File I/O**: Same as normal debug mode

## Common Use Cases

### 1. Local Development

Test your data transformation logic without needing Databricks access:

```rust
let config = WrapperConfiguration::new(
    "https://workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_debug_output(PathBuf::from("./debug_output"))
.with_zerobus_writer_disabled(true);
```

### 2. Unit Testing

Test conversion logic in unit tests:

```rust
#[tokio::test]
async fn test_conversion() {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(PathBuf::from("./test_debug"))
    .with_zerobus_writer_disabled(true);
    
    let wrapper = ZerobusWrapper::new(config).await.unwrap();
    let result = wrapper.send_batch(test_batch()).await.unwrap();
    assert!(result.success);
}
```

### 3. CI/CD Validation

Validate data format in CI/CD without credentials:

```python
wrapper = ZerobusWrapper(
    endpoint="https://test.cloud.databricks.com",
    table_name="test_table",
    debug_enabled=True,
    debug_output_dir="./ci_debug",
    zerobus_writer_disabled=True,
)
```

## Troubleshooting

### Error: "debug_enabled must be true when zerobus_writer_disabled is true"

**Solution**: Enable debug output:

```rust
.with_debug_output(PathBuf::from("./debug_output"))
// or
.with_debug_enabled(true)  // If using direct field access
```

### Debug Files Not Created

**Check**:
1. Is `debug_enabled` set to `true`?
2. Is `debug_output_dir` set and writable?
3. Did conversion succeed? (Check `result.success`)

### Want to Actually Send Data

**Solution**: Set `zerobus_writer_disabled` to `false` (default) and provide credentials:

```rust
let config = WrapperConfiguration::new(
    "https://workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_credentials("client_id".to_string(), "client_secret".to_string())
.with_unity_catalog("https://unity-catalog-url".to_string())
.with_zerobus_writer_disabled(false);  // Or omit (default is false)
```

## Next Steps

- Review the [API contracts](./contracts/) for detailed API documentation
- Check the [data model](./data-model.md) for configuration details
- See the [full specification](./spec.md) for complete feature documentation


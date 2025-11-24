# Quickstart Guide: Zerobus SDK Wrapper

**Feature**: 001-zerobus-wrapper  
**Date**: 2025-11-23

## Overview

This guide provides quick examples for using the Zerobus SDK Wrapper from both Rust and Python applications to send Arrow RecordBatch data to Databricks Zerobus.

## Prerequisites

- Rust 1.75+ (for Rust usage)
- Python 3.11+ (for Python usage)
- Databricks workspace with Zerobus enabled
- OAuth2 credentials (client_id, client_secret)
- Unity Catalog URL

## Rust Quickstart

### 1. Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
arrow-zerobus-sdk-wrapper = { version = "0.1.0", path = "../arrow-zerobus-sdk-wrapper" }
arrow = "57"
tokio = { version = "1.35", features = ["full"] }
```

### 2. Basic Usage

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
use arrow::array::{Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use arrow::datatypes::{Schema, Field, DataType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = WrapperConfiguration::new(
        "https://your-workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    )
    .with_credentials(
        std::env::var("ZEROBUS_CLIENT_ID")?,
        std::env::var("ZEROBUS_CLIENT_SECRET")?,
    )
    .with_unity_catalog(
        std::env::var("UNITY_CATALOG_URL")?,
    );
    
    // Initialize wrapper
    let wrapper = ZerobusWrapper::new(config).await?;
    
    // Create Arrow RecordBatch
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    
    let id_array = Int64Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);
    
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )?;
    
    // Send batch
    let result = wrapper.send_batch(batch).await?;
    
    if result.success {
        println!("✅ Batch sent successfully!");
        println!("   Latency: {}ms", result.latency_ms.unwrap_or(0));
        println!("   Size: {} bytes", result.batch_size_bytes);
    } else {
        eprintln!("❌ Transmission failed: {:?}", result.error);
    }
    
    // Shutdown
    wrapper.shutdown().await?;
    
    Ok(())
}
```

### 3. With Observability

```rust
use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration, OtlpConfig};

let config = WrapperConfiguration::new(
    "https://your-workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_credentials(client_id, client_secret)
.with_unity_catalog(unity_catalog_url)
.with_observability(OtlpConfig {
    // Configure OpenTelemetry export
    endpoint: Some("http://localhost:4317".to_string()),
    // ... other OTLP config
});

let wrapper = ZerobusWrapper::new(config).await?;
```

### 4. With Debug Output

```rust
use std::path::PathBuf;

let config = WrapperConfiguration::new(
    "https://your-workspace.cloud.databricks.com".to_string(),
    "my_table".to_string(),
)
.with_credentials(client_id, client_secret)
.with_unity_catalog(unity_catalog_url)
.with_debug_output(PathBuf::from("./debug_output"))
// Debug files will be written to ./debug_output/zerobus/arrow/ and ./debug_output/zerobus/proto/
```

## Python Quickstart

### 1. Installation

```bash
pip install arrow-zerobus-sdk-wrapper
# Or from local development:
pip install -e /path/to/arrow-zerobus-sdk-wrapper
```

### 2. Basic Usage

```python
import asyncio
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper
import os

async def main():
    # Initialize wrapper
    wrapper = ZerobusWrapper(
        endpoint=os.getenv("ZEROBUS_ENDPOINT"),
        table_name="my_table",
        client_id=os.getenv("ZEROBUS_CLIENT_ID"),
        client_secret=os.getenv("ZEROBUS_CLIENT_SECRET"),
        unity_catalog_url=os.getenv("UNITY_CATALOG_URL"),
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
        print(f"✅ Batch sent successfully!")
        print(f"   Latency: {result.latency_ms}ms")
        print(f"   Size: {result.batch_size_bytes} bytes")
    else:
        print(f"❌ Transmission failed: {result.error}")
    
    # Shutdown
    await wrapper.shutdown()

if __name__ == "__main__":
    asyncio.run(main())
```

### 3. Using Async Context Manager

```python
async def main():
    async with ZerobusWrapper(
        endpoint=os.getenv("ZEROBUS_ENDPOINT"),
        table_name="my_table",
        client_id=os.getenv("ZEROBUS_CLIENT_ID"),
        client_secret=os.getenv("ZEROBUS_CLIENT_SECRET"),
        unity_catalog_url=os.getenv("UNITY_CATALOG_URL"),
    ) as wrapper:
        batch = create_record_batch()
        result = await wrapper.send_batch(batch)
        print(f"Result: {result.success}")
```

### 4. With Observability

```python
wrapper = ZerobusWrapper(
    endpoint=endpoint,
    table_name=table_name,
    client_id=client_id,
    client_secret=client_secret,
    unity_catalog_url=unity_catalog_url,
    observability_enabled=True,
    observability_config={
        "endpoint": "http://localhost:4317",
        # ... other OTLP config
    },
)
```

### 5. With Debug Output

```python
wrapper = ZerobusWrapper(
    endpoint=endpoint,
    table_name=table_name,
    client_id=client_id,
    client_secret=client_secret,
    unity_catalog_url=unity_catalog_url,
    debug_enabled=True,
    debug_output_dir="./debug_output",
    debug_flush_interval_secs=5,
    debug_max_file_size=100 * 1024 * 1024,  # 100MB
)
```

## Error Handling

### Rust

```rust
match wrapper.send_batch(batch).await {
    Ok(result) => {
        if result.success {
            println!("Success!");
        } else {
            eprintln!("Failed: {:?}", result.error);
        }
    }
    Err(e) => {
        match e {
            ZerobusError::ConfigurationError(msg) => {
                eprintln!("Configuration error: {}", msg);
            }
            ZerobusError::AuthenticationError(msg) => {
                eprintln!("Authentication error: {}", msg);
            }
            ZerobusError::RetryExhausted(msg) => {
                eprintln!("All retries exhausted: {}", msg);
            }
            _ => {
                eprintln!("Error: {:?}", e);
            }
        }
    }
}
```

### Python

```python
try:
    result = await wrapper.send_batch(batch)
    if result.success:
        print("Success!")
    else:
        print(f"Failed: {result.error}")
except ZerobusError as e:
    if isinstance(e, ConfigurationError):
        print(f"Configuration error: {e}")
    elif isinstance(e, AuthenticationError):
        print(f"Authentication error: {e}")
    elif isinstance(e, RetryExhausted):
        print(f"All retries exhausted: {e}")
    else:
        print(f"Error: {e}")
```

## Configuration Options

### Retry Configuration

```rust
// Rust
let config = WrapperConfiguration::new(endpoint, table_name)
    .with_retry_config(
        max_attempts: 10,      // Maximum retry attempts
        base_delay_ms: 200,    // Base delay for exponential backoff
        max_delay_ms: 60000,   // Maximum delay (60 seconds)
    );
```

```python
# Python
wrapper = ZerobusWrapper(
    endpoint=endpoint,
    table_name=table_name,
    retry_max_attempts=10,
    retry_base_delay_ms=200,
    retry_max_delay_ms=60000,
)
```

## Testing

### Rust

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_batch() {
        let config = create_test_config();
        let wrapper = ZerobusWrapper::new(config).await.unwrap();
        let batch = create_test_batch();
        
        let result = wrapper.send_batch(batch).await.unwrap();
        assert!(result.success);
    }
}
```

### Python

```python
import pytest

@pytest.mark.asyncio
async def test_send_batch():
    wrapper = ZerobusWrapper(
        endpoint="https://test-endpoint",
        table_name="test_table",
        client_id="test_id",
        client_secret="test_secret",
        unity_catalog_url="https://test-catalog",
    )
    
    batch = create_test_batch()
    result = await wrapper.send_batch(batch)
    
    assert result.success
```

## Next Steps

- See [Rust API Contract](./contracts/rust-api.md) for detailed API documentation
- See [Python API Contract](./contracts/python-api.md) for detailed API documentation
- See [Data Model](./data-model.md) for entity definitions
- See [Implementation Plan](./plan.md) for technical details


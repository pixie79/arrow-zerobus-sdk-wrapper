# Python API Contract: Zerobus SDK Wrapper

**Feature**: 001-zerobus-wrapper  
**Date**: 2025-11-23  
**Version**: 0.1.0

## Public API

### ZerobusWrapper

Main wrapper class for sending data to Zerobus.

```python
class ZerobusWrapper:
    """Wrapper for sending Arrow RecordBatch data to Databricks Zerobus.
    
    Thread-safe and supports concurrent operations from multiple threads.
    """
    
    def __init__(
        self,
        endpoint: str,
        table_name: str,
        *,
        client_id: Optional[str] = None,
        client_secret: Optional[str] = None,
        unity_catalog_url: Optional[str] = None,
        observability_enabled: bool = False,
        observability_config: Optional[Dict[str, Any]] = None,
        debug_enabled: bool = False,
        debug_output_dir: Optional[str] = None,
        debug_flush_interval_secs: int = 5,
        debug_max_file_size: Optional[int] = None,
        retry_max_attempts: int = 5,
        retry_base_delay_ms: int = 100,
        retry_max_delay_ms: int = 30000,
    ) -> None:
        """Initialize ZerobusWrapper with configuration.
        
        Args:
            endpoint: Zerobus endpoint URL (required)
            table_name: Target table name (required)
            client_id: OAuth2 client ID (required for authentication)
            client_secret: OAuth2 client secret (required for authentication)
            unity_catalog_url: Unity Catalog URL (required for SDK)
            observability_enabled: Enable OpenTelemetry observability
            observability_config: OpenTelemetry configuration dict
            debug_enabled: Enable debug file output
            debug_output_dir: Output directory for debug files
            debug_flush_interval_secs: Debug file flush interval in seconds
            debug_max_file_size: Maximum debug file size before rotation
            retry_max_attempts: Maximum retry attempts for transient failures
            retry_base_delay_ms: Base delay in milliseconds for exponential backoff
            retry_max_delay_ms: Maximum delay in milliseconds for exponential backoff
        
        Raises:
            ZerobusError: If configuration is invalid or initialization fails
        """
    
    async def send_batch(self, batch: pyarrow.RecordBatch) -> TransmissionResult:
        """Send an Arrow RecordBatch to Zerobus.
        
        Args:
            batch: PyArrow RecordBatch to send
        
        Returns:
            TransmissionResult indicating success or failure
        
        Raises:
            ZerobusError: If transmission fails after all retry attempts
        """
    
    async def flush(self) -> None:
        """Flush any pending operations and ensure data is transmitted.
        
        Raises:
            ZerobusError: If flush operation fails
        """
    
    async def shutdown(self) -> None:
        """Shutdown the wrapper gracefully, closing connections and cleaning up resources.
        
        Raises:
            ZerobusError: If shutdown fails
        """
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    async def __aenter__(self):
        """Async context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        # Synchronous shutdown if needed
        pass
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.shutdown()
```

### TransmissionResult

Result of a data transmission operation.

```python
@dataclass
class TransmissionResult:
    """Result of a data transmission operation."""
    
    success: bool
    """Whether transmission succeeded."""
    
    error: Optional[str]
    """Error message if transmission failed."""
    
    attempts: int
    """Number of retry attempts made."""
    
    latency_ms: Optional[int]
    """Transmission latency in milliseconds (if successful)."""
    
    batch_size_bytes: int
    """Size of transmitted batch in bytes."""
```

### ZerobusError

Exception type for wrapper operations.

```python
class ZerobusError(Exception):
    """Base exception for Zerobus wrapper errors."""
    pass

class ConfigurationError(ZerobusError):
    """Invalid configuration error."""
    pass

class AuthenticationError(ZerobusError):
    """Authentication failure error."""
    pass

class ConnectionError(ZerobusError):
    """Network/connection error."""
    pass

class ConversionError(ZerobusError):
    """Arrow to Protobuf conversion failure."""
    pass

class TransmissionError(ZerobusError):
    """Data transmission failure."""
    pass

class RetryExhausted(ZerobusError):
    """All retry attempts exhausted."""
    pass

class TokenRefreshError(ZerobusError):
    """Token refresh failure."""
    pass
```

## Usage Example

```python
import asyncio
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def main():
    # Initialize wrapper
    wrapper = ZerobusWrapper(
        endpoint="https://workspace.cloud.databricks.com",
        table_name="my_table",
        client_id="client_id",
        client_secret="client_secret",
        unity_catalog_url="https://unity-catalog-url",
        debug_enabled=True,
        debug_output_dir="./debug_output",
    )
    
    # Create Arrow RecordBatch (example)
    schema = pa.schema([
        pa.field("id", pa.int64()),
        pa.field("name", pa.string()),
    ])
    arrays = [
        pa.array([1, 2, 3], type=pa.int64()),
        pa.array(["a", "b", "c"], type=pa.string()),
    ]
    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
    
    # Send batch
    result = await wrapper.send_batch(batch)
    
    if result.success:
        print(f"Batch sent successfully in {result.latency_ms}ms")
    else:
        print(f"Transmission failed: {result.error}")
    
    # Shutdown
    await wrapper.shutdown()

# Using async context manager
async def main_with_context():
    async with ZerobusWrapper(
        endpoint="https://workspace.cloud.databricks.com",
        table_name="my_table",
        client_id="client_id",
        client_secret="client_secret",
    ) as wrapper:
        batch = create_record_batch()
        result = await wrapper.send_batch(batch)
        print(f"Result: {result.success}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Thread Safety

All public methods are thread-safe and can be called concurrently from multiple threads or async tasks. The wrapper uses internal synchronization to ensure safe concurrent access.

## Error Handling

All errors are raised as `ZerobusError` or its subclasses. Errors are descriptive and actionable, providing sufficient information for developers to diagnose and resolve issues.

## Performance Characteristics

- **Latency**: p95 latency under 150ms for batches up to 10MB
- **Success Rate**: 99.999% under normal network conditions
- **Concurrency**: Thread-safe, supports concurrent operations
- **Memory**: Bounded memory usage per wrapper instance

## Dependencies

- Python 3.11+
- PyArrow (for Arrow RecordBatch support)
- asyncio (for async operations)

## Installation

```bash
pip install arrow-zerobus-sdk-wrapper
```

## Versioning

The API follows semantic versioning. Breaking changes will be indicated by major version bumps and will include migration guides.


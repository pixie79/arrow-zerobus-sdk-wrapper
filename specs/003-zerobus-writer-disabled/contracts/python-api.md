# Python API Contract: Zerobus Writer Disabled Mode

**Feature**: 003-zerobus-writer-disabled  
**Date**: 2025-12-11  
**Version**: 0.1.0 (additive change)

## API Changes

This document describes the API changes for the Zerobus Writer Disabled Mode feature. All existing APIs remain unchanged; this feature adds a new configuration parameter.

### ZerobusWrapper.__init__ (Modified)

Constructor updated to include writer disabled mode parameter.

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
        zerobus_writer_disabled: bool = False,  # NEW PARAMETER
    ) -> None:
        """Initialize ZerobusWrapper with configuration.
        
        Args:
            endpoint: Zerobus endpoint URL (required)
            table_name: Target table name (required)
            client_id: OAuth2 client ID (optional when zerobus_writer_disabled is True)
            client_secret: OAuth2 client secret (optional when zerobus_writer_disabled is True)
            unity_catalog_url: Unity Catalog URL (optional when zerobus_writer_disabled is True)
            observability_enabled: Enable OpenTelemetry observability
            observability_config: OpenTelemetry configuration dict
            debug_enabled: Enable debug file output (required when zerobus_writer_disabled is True)
            debug_output_dir: Output directory for debug files
            debug_flush_interval_secs: Debug file flush interval in seconds
            debug_max_file_size: Maximum debug file size before rotation
            retry_max_attempts: Maximum retry attempts for transient failures
            retry_base_delay_ms: Base delay in milliseconds for exponential backoff
            retry_max_delay_ms: Maximum delay in milliseconds for exponential backoff
            zerobus_writer_disabled: Disable Zerobus SDK transmission while maintaining debug output
        
        Raises:
            ZerobusError: If configuration is invalid or initialization fails
                - ConfigurationError if zerobus_writer_disabled is True but debug_enabled is False
        """
```

### Validation Behavior

**New Validation Rule**:
- If `zerobus_writer_disabled` is `True` and `debug_enabled` is `False`, initialization raises:
  ```python
  ZerobusError("Configuration error: debug_enabled must be true when zerobus_writer_disabled is true")
  ```

**Modified Validation Rule**:
- When `zerobus_writer_disabled` is `True`, credential parameters (`client_id`, `client_secret`, `unity_catalog_url`) are optional (not required)

### send_batch (Behavior Change)

No API signature changes, but behavior changes when `zerobus_writer_disabled` is `True`:

```python
async def send_batch(self, batch: pyarrow.RecordBatch) -> TransmissionResult:
    """Send an Arrow RecordBatch to Zerobus.
    
    # Behavior When Writer Disabled
    
    When `zerobus_writer_disabled` is `True`:
    - Debug files (Arrow and Protobuf) are written normally
    - Arrow-to-Protobuf conversion executes normally
    - Zerobus SDK initialization is skipped
    - Stream creation is skipped
    - Data transmission calls are skipped
    - Returns `TransmissionResult` with `success=True` if conversion succeeds
    - Returns `TransmissionResult` with `success=False` if conversion fails
    
    Args:
        batch: PyArrow RecordBatch to send
    
    Returns:
        TransmissionResult indicating success or failure
    
    Raises:
        ZerobusError: If conversion fails (when writer disabled) or transmission fails (when writer enabled)
    """
```

### TransmissionResult (No Changes)

No structural changes. Behavior unchanged - returns success when writer is disabled and conversion succeeds.

```python
class TransmissionResult:
    """Result of a data transmission operation."""
    
    success: bool
    error: Optional[ZerobusError]
    attempts: int
    latency_ms: Optional[int]
    batch_size_bytes: int
```

**Behavior Notes**:
- When writer is disabled: `success` is `True` if conversion succeeds, `False` if conversion fails
- `attempts` will be `1` (no retry logic executed when disabled)
- `latency_ms` reflects conversion time only (no network overhead)

## Error Types

### ZerobusError (No Changes)

No new error types. Existing `ConfigurationError` is used for validation failures.

## Examples

### Example 1: Local Development Without Network

```python
import asyncio
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def main():
    wrapper = ZerobusWrapper(
        endpoint="https://workspace.cloud.databricks.com",
        table_name="my_table",
        debug_enabled=True,
        debug_output_dir="./debug_output",
        zerobus_writer_disabled=True,  # Disable transmission
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
    
    # Send batch - will write debug files but skip network calls
    result = await wrapper.send_batch(batch)
    
    # Result will be success (conversion succeeded)
    assert result.success
    
    await wrapper.shutdown()

asyncio.run(main())
```

### Example 2: CI/CD Testing Without Credentials

```python
import os
import pyarrow as pa
from arrow_zerobus_sdk_wrapper import ZerobusWrapper

async def test_data_conversion():
    """Test data conversion without requiring Databricks credentials."""
    wrapper = ZerobusWrapper(
        endpoint=os.getenv("ZEROBUS_ENDPOINT", "https://test.cloud.databricks.com"),
        table_name="test_table",
        debug_enabled=True,
        debug_output_dir=os.getenv("DEBUG_OUTPUT_DIR", "./test_debug"),
        zerobus_writer_disabled=True,  # No credentials needed
    )
    
    # Test batch conversion
    batch = create_test_batch()
    result = await wrapper.send_batch(batch)
    
    # Verify conversion succeeded
    assert result.success
    
    # Verify debug files were written
    assert os.path.exists("./test_debug/zerobus/arrow/test_table.arrow")
    assert os.path.exists("./test_debug/zerobus/proto/test_table.proto")
    
    await wrapper.shutdown()
```

### Example 3: Configuration Validation Error

```python
from arrow_zerobus_sdk_wrapper import ZerobusWrapper, ZerobusError

try:
    wrapper = ZerobusWrapper(
        endpoint="https://workspace.cloud.databricks.com",
        table_name="my_table",
        debug_enabled=False,  # Missing debug output
        zerobus_writer_disabled=True,  # But requires debug
    )
except ZerobusError as e:
    # Will raise: "Configuration error: debug_enabled must be true when zerobus_writer_disabled is true"
    print(f"Configuration error: {e}")
```

## Backward Compatibility

- ✅ All existing APIs remain unchanged
- ✅ Default value for `zerobus_writer_disabled` is `False` (existing behavior preserved)
- ✅ No breaking changes to existing code
- ✅ Optional parameter - existing code continues to work without modification

## Migration Guide

No migration required. This is an additive feature. To use:

1. Add `zerobus_writer_disabled=True` to your `ZerobusWrapper` constructor
2. Ensure `debug_enabled=True` (or set `debug_output_dir`)
3. Credentials become optional when writer is disabled
4. Existing code continues to work without changes


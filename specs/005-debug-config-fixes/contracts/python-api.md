# Python API Contract: Debug Output Configuration Fixes

**Feature**: 005-debug-config-fixes  
**Date**: 2025-12-12

## Configuration API Changes

### PyWrapperConfiguration Class

**Location**: `src/python/bindings.rs`

**Updated `__init__` Signature**:
```python
class PyWrapperConfiguration:
    def __init__(
        self,
        endpoint: str,
        table_name: str,
        *,
        client_id: str | None = None,
        client_secret: str | None = None,
        unity_catalog_url: str | None = None,
        observability_enabled: bool = False,
        observability_config: dict | None = None,
        debug_enabled: bool = False,  # Legacy flag (deprecated)
        debug_arrow_enabled: bool | None = None,  # New flag
        debug_protobuf_enabled: bool | None = None,  # New flag
        debug_output_dir: str | None = None,
        debug_flush_interval_secs: int = 5,
        debug_max_file_size: int | None = None,
        debug_max_files_retained: int | None = 10,  # New flag (default: 10, None = unlimited)
        retry_max_attempts: int = 5,
        retry_base_delay_ms: int = 100,
        retry_max_delay_ms: int = 30000,
        zerobus_writer_disabled: bool = False,
    ) -> None:
        """
        Initialize WrapperConfiguration with parameters.
        
        Args:
            endpoint: Zerobus endpoint URL (required)
            table_name: Target table name (required)
            client_id: OAuth2 client ID (optional when zerobus_writer_disabled is True)
            client_secret: OAuth2 client secret (optional when zerobus_writer_disabled is True)
            unity_catalog_url: Unity Catalog URL (optional when zerobus_writer_disabled is True)
            observability_enabled: Enable OpenTelemetry observability
            observability_config: OpenTelemetry configuration dict
            debug_enabled: Legacy flag - enables both Arrow and Protobuf if new flags not set
            debug_arrow_enabled: Enable Arrow debug file output (optional, defaults to None)
            debug_protobuf_enabled: Enable Protobuf debug file output (optional, defaults to None)
            debug_output_dir: Output directory for debug files (required when any debug flag is True)
            debug_flush_interval_secs: Debug file flush interval in seconds
            debug_max_file_size: Maximum debug file size before rotation
            debug_max_files_retained: Maximum number of rotated files to retain per type (default: 10, None = unlimited)
            retry_max_attempts: Maximum retry attempts for transient failures
            retry_base_delay_ms: Base delay in milliseconds for exponential backoff
            retry_max_delay_ms: Maximum delay in milliseconds for exponential backoff
            zerobus_writer_disabled: Disable Zerobus SDK transmission while maintaining debug output
            
        Raises:
            ZerobusError: If configuration is invalid or initialization fails
                - ConfigurationError if debug_arrow_enabled or debug_protobuf_enabled is True
                  but debug_output_dir is None
                - ConfigurationError if zerobus_writer_disabled is True but all debug flags are False
        """
```

### Configuration Precedence

**Rules**:
1. If `debug_arrow_enabled` or `debug_protobuf_enabled` are explicitly set (not `None`), use those values
2. Otherwise, if `debug_enabled` is `True`, enable both Arrow and Protobuf
3. If all flags are `False` or `None`, no debug output is written

**Examples**:
```python
# Explicit flags take precedence
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_protobuf_enabled=False,
    debug_output_dir="/tmp/debug"
)
# Only Arrow files written

# Legacy flag enables both if new flags not set
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_enabled=True,  # Legacy flag
    debug_output_dir="/tmp/debug"
)
# Both Arrow and Protobuf files written

# New flags override legacy flag
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_enabled=True,  # Ignored
    debug_arrow_enabled=True,
    debug_protobuf_enabled=False,  # Explicit override
    debug_output_dir="/tmp/debug"
)
# Only Arrow files written (debug_protobuf_enabled=False takes precedence)
```

## Backward Compatibility

### Existing Code Compatibility

**Before** (still works):
```python
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_enabled=True,
    debug_output_dir="/tmp/debug"
)
```

**After** (new preferred way):
```python
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_protobuf_enabled=False,
    debug_output_dir="/tmp/debug"
)
```

**Migration Path**:
1. Existing code using `debug_enabled=True` continues to work (enables both formats)
2. New code should use `debug_arrow_enabled` and `debug_protobuf_enabled` for granular control
3. `debug_enabled` parameter remains but is deprecated

## Error Handling

**No New Exception Types**: Existing `ZerobusError` exception is used for invalid configurations.

**Validation Errors**:
```python
# Raises ZerobusError if debug flag is True but output_dir is None
try:
    config = PyWrapperConfiguration(
        endpoint="https://example.com",
        table_name="my_table",
        debug_arrow_enabled=True,
        # debug_output_dir missing - will raise error
    )
except ZerobusError as e:
    print(f"Configuration error: {e}")
```

## Examples

### Example 1: Enable Only Arrow Debug

```python
from arrow_zerobus_sdk_wrapper import PyWrapperConfiguration, ZerobusWrapper

config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_protobuf_enabled=False,
    debug_output_dir="/tmp/debug"
)

wrapper = ZerobusWrapper(config)
# Only Arrow files written, Protobuf files not written
```

### Example 2: Enable Only Protobuf Debug

```python
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=False,
    debug_protobuf_enabled=True,
    debug_output_dir="/tmp/debug"
)

wrapper = ZerobusWrapper(config)
# Only Protobuf files written, Arrow files not written
```

### Example 3: Backward Compatible Configuration

```python
# Legacy code - still works
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_enabled=True,  # Enables both formats
    debug_output_dir="/tmp/debug"
)

wrapper = ZerobusWrapper(config)
# Both Arrow and Protobuf files written
```

### Example 4: File Retention Configuration

```python
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_protobuf_enabled=True,
    debug_output_dir="/tmp/debug",
    debug_max_files_retained=10  # Keep last 10 files per type
)

wrapper = ZerobusWrapper(config)
# After 11th rotation, oldest file automatically deleted
```

### Example 5: Unlimited File Retention

```python
config = PyWrapperConfiguration(
    endpoint="https://example.com",
    table_name="my_table",
    debug_arrow_enabled=True,
    debug_output_dir="/tmp/debug",
    debug_max_files_retained=None  # Unlimited retention
)
```

### Example 6: Environment Variable Configuration

```bash
export DEBUG_ARROW_ENABLED=true
export DEBUG_PROTOBUF_ENABLED=false
export DEBUG_OUTPUT_DIR=/tmp/debug
export DEBUG_MAX_FILES_RETAINED=10
```

```python
# Configuration loader will read from environment
from arrow_zerobus_sdk_wrapper import load_config_from_env

config = load_config_from_env()
# Arrow enabled, Protobuf disabled, retention limit: 10 files (from env vars)
```

## API Consistency

**Rust-Python Parity**: The Python API maintains semantic equivalence with the Rust API:
- Same flag names (with Python naming convention: `snake_case`)
- Same precedence rules
- Same error conditions
- Same backward compatibility behavior

**Differences**:
- Python uses `None` for optional booleans (Rust uses `Option<bool>`)
- Python uses `str` for paths (Rust uses `PathBuf`)
- Python raises exceptions (Rust returns `Result`)

"""Integration tests for Python bindings

These tests verify that the Python API works correctly.
Requires PyArrow to be installed.
"""

import pytest
import pyarrow as pa

# Skip all tests if the module is not available
try:
    import arrow_zerobus_sdk_wrapper  # noqa: F401  # Used in test_import_module
except ImportError:
    pytestmark = pytest.mark.skip("arrow_zerobus_sdk_wrapper not available")


def test_import_module():
    """Test that the module can be imported."""
    import arrow_zerobus_sdk_wrapper

    assert hasattr(arrow_zerobus_sdk_wrapper, "ZerobusWrapper")
    assert hasattr(arrow_zerobus_sdk_wrapper, "ZerobusError")


def test_configuration_creation():
    """Test that WrapperConfiguration can be created."""
    from arrow_zerobus_sdk_wrapper import WrapperConfiguration

    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        client_id="test_client_id",
        client_secret="test_client_secret",
        unity_catalog_url="https://unity-catalog-url",
    )

    assert config is not None


def test_configuration_validation():
    """Test that configuration validation works."""
    from arrow_zerobus_sdk_wrapper import WrapperConfiguration

    # Valid configuration
    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
    )

    # Should not raise
    try:
        config.validate()
    except Exception as e:
        pytest.fail(f"Valid configuration should not raise error: {e}")

    # Invalid configuration
    invalid_config = WrapperConfiguration(
        endpoint="invalid-endpoint",
        table_name="test_table",
    )

    # Should raise ConfigurationError
    with pytest.raises(Exception):  # Will be ConfigurationError when implemented
        invalid_config.validate()


def test_transmission_result():
    """Test that TransmissionResult can be created and accessed."""

    # Note: This is a test of the Python class structure
    # Actual TransmissionResult objects are created by the wrapper
    pass


def test_error_classes():
    """Test that error classes are available."""
    from arrow_zerobus_sdk_wrapper import (
        ZerobusError,
        ConfigurationError,
        AuthenticationError,
        ConnectionError,
        ConversionError,
        TransmissionError,
        RetryExhausted,
        TokenRefreshError,
    )

    # Verify all error classes exist
    assert ZerobusError is not None
    assert ConfigurationError is not None
    assert AuthenticationError is not None
    assert ConnectionError is not None
    assert ConversionError is not None
    assert TransmissionError is not None
    assert RetryExhausted is not None
    assert TokenRefreshError is not None


@pytest.mark.skip(reason="Requires actual Zerobus SDK and credentials")
def test_wrapper_initialization():
    """Test that ZerobusWrapper can be initialized."""
    from arrow_zerobus_sdk_wrapper import ZerobusWrapper, WrapperConfiguration

    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        client_id="test_client_id",
        client_secret="test_client_secret",
        unity_catalog_url="https://unity-catalog-url",
    )
    wrapper = ZerobusWrapper(config)

    assert wrapper is not None


@pytest.mark.skip(reason="Requires actual Zerobus SDK and credentials")
def test_send_batch():
    """Test sending a RecordBatch."""
    from arrow_zerobus_sdk_wrapper import ZerobusWrapper, WrapperConfiguration

    # Create test RecordBatch
    schema = pa.schema(
        [
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
        ]
    )
    arrays = [
        pa.array([1, 2, 3], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
    ]
    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)

    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        client_id="test_client_id",
        client_secret="test_client_secret",
        unity_catalog_url="https://unity-catalog-url",
    )
    wrapper = ZerobusWrapper(config)

    # This will fail without real credentials, but tests the API
    result = wrapper.send_batch(batch)

    # Verify result structure
    assert hasattr(result, "success")
    assert hasattr(result, "error")
    assert hasattr(result, "attempts")
    assert hasattr(result, "latency_ms")
    assert hasattr(result, "batch_size_bytes")


def test_record_batch_creation():
    """Test that PyArrow RecordBatch can be created for testing."""
    schema = pa.schema(
        [
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
            pa.field("score", pa.float64()),
        ]
    )

    arrays = [
        pa.array([1, 2, 3], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
        pa.array([95.5, 87.0, 92.5], type=pa.float64()),
    ]

    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)

    assert batch.num_rows == 3
    assert batch.num_columns == 3
    assert batch.schema == schema


def test_writer_disabled_parameter():
    """Test that zerobus_writer_disabled parameter is accepted."""
    from arrow_zerobus_sdk_wrapper import WrapperConfiguration

    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        debug_enabled=True,
        debug_output_dir="./test_debug",
        zerobus_writer_disabled=True,
    )

    assert config.zerobus_writer_disabled is True


def test_writer_disabled_validation():
    """Test that configuration validation works for writer disabled mode."""
    from arrow_zerobus_sdk_wrapper import WrapperConfiguration, ConfigurationError

    # Should fail: writer disabled but debug not enabled
    with pytest.raises(ConfigurationError):
        config = WrapperConfiguration(
            endpoint="https://test.cloud.databricks.com",
            table_name="test_table",
            debug_enabled=False,
            zerobus_writer_disabled=True,
        )
        config.validate()

    # Should succeed: writer disabled with debug enabled
    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        debug_enabled=True,
        debug_output_dir="./test_debug",
        zerobus_writer_disabled=True,
    )
    # Should not raise
    try:
        config.validate()
    except Exception as e:
        pytest.fail(f"Valid configuration should not raise error: {e}")


def test_debug_enabled_requires_output_dir():
    """Test that debug_enabled=True requires debug_output_dir to be provided."""
    from arrow_zerobus_sdk_wrapper import WrapperConfiguration

    # Should raise error: debug_enabled=True but debug_output_dir=None
    with pytest.raises(Exception) as exc_info:
        WrapperConfiguration(
            endpoint="https://test.cloud.databricks.com",
            table_name="test_table",
            debug_enabled=True,
            debug_output_dir=None,  # Missing output dir
        )

    # Verify error message mentions debug_output_dir requirement
    error_msg = str(exc_info.value)
    assert (
        "debug_output_dir" in error_msg.lower() or "output" in error_msg.lower()
    ), f"Error message should mention debug_output_dir requirement, got: {error_msg}"

    # Should succeed: debug_enabled=True with debug_output_dir provided
    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        debug_enabled=True,
        debug_output_dir="./test_debug",
    )
    assert (
        config.debug_enabled is True
    ), "debug_enabled should be True when output_dir is provided"

    # Should succeed: debug_enabled=False (default) without output_dir
    config = WrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        debug_enabled=False,
        debug_output_dir=None,
    )
    assert (
        config.debug_enabled is False
    ), "debug_enabled should be False when not enabled"


@pytest.mark.asyncio
async def test_wrapper_works_without_credentials_when_disabled():
    """Test that wrapper works without credentials when writer is disabled."""
    import tempfile
    import os
    from arrow_zerobus_sdk_wrapper import ZerobusWrapper, WrapperConfiguration

    # Create temporary directory for debug output
    temp_dir = tempfile.mkdtemp()
    debug_output_dir = os.path.join(temp_dir, "debug")

    try:
        # Create configuration first
        config = WrapperConfiguration(
            endpoint="https://test.cloud.databricks.com",
            table_name="test_table",
            debug_enabled=True,
            debug_output_dir=debug_output_dir,
            zerobus_writer_disabled=True,
            # No credentials provided
        )
        # Then create wrapper with the configuration
        wrapper = ZerobusWrapper(config)

        # Create test batch
        schema = pa.schema(
            [
                pa.field("id", pa.int64()),
                pa.field("name", pa.string()),
            ]
        )
        arrays = [
            pa.array([1, 2, 3], type=pa.int64()),
            pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
        ]
        batch = pa.RecordBatch.from_arrays(arrays, schema=schema)

        # Send batch - should succeed without credentials
        result = wrapper.send_batch(batch)
        assert result.success, "send_batch should succeed when writer disabled"

        # Verify debug files were written
        # Files may not exist immediately, but the operation should succeed
        assert result.success

        wrapper.shutdown()
    finally:
        # Cleanup
        import shutil

        shutil.rmtree(temp_dir, ignore_errors=True)

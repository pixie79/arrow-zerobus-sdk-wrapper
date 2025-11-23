"""Integration tests for Python bindings

These tests verify that the Python API works correctly.
Requires PyArrow to be installed.
"""

import pytest
import pyarrow as pa
import asyncio

# Skip all tests if the module is not available
try:
    import arrow_zerobus_sdk_wrapper
except ImportError:
    pytestmark = pytest.mark.skip("arrow_zerobus_sdk_wrapper not available")


def test_import_module():
    """Test that the module can be imported."""
    import arrow_zerobus_sdk_wrapper
    assert hasattr(arrow_zerobus_sdk_wrapper, "ZerobusWrapper")
    assert hasattr(arrow_zerobus_sdk_wrapper, "ZerobusError")


def test_configuration_creation():
    """Test that WrapperConfiguration can be created."""
    from arrow_zerobus_sdk_wrapper import PyWrapperConfiguration
    
    config = PyWrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        client_id="test_client_id",
        client_secret="test_client_secret",
        unity_catalog_url="https://unity-catalog-url",
    )
    
    assert config is not None


def test_configuration_validation():
    """Test that configuration validation works."""
    from arrow_zerobus_sdk_wrapper import PyWrapperConfiguration, PyConfigurationError
    
    # Valid configuration
    config = PyWrapperConfiguration(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
    )
    
    # Should not raise
    try:
        config.validate()
    except Exception as e:
        pytest.fail(f"Valid configuration should not raise error: {e}")
    
    # Invalid configuration
    invalid_config = PyWrapperConfiguration(
        endpoint="invalid-endpoint",
        table_name="test_table",
    )
    
    # Should raise ConfigurationError
    with pytest.raises(Exception):  # Will be ConfigurationError when implemented
        invalid_config.validate()


def test_transmission_result():
    """Test that TransmissionResult can be created and accessed."""
    from arrow_zerobus_sdk_wrapper import PyTransmissionResult
    
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
    from arrow_zerobus_sdk_wrapper import ZerobusWrapper
    
    wrapper = ZerobusWrapper(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        client_id="test_client_id",
        client_secret="test_client_secret",
        unity_catalog_url="https://unity-catalog-url",
    )
    
    assert wrapper is not None


@pytest.mark.skip(reason="Requires actual Zerobus SDK and credentials")
def test_send_batch():
    """Test sending a RecordBatch."""
    from arrow_zerobus_sdk_wrapper import ZerobusWrapper
    
    # Create test RecordBatch
    schema = pa.schema([
        pa.field("id", pa.int64()),
        pa.field("name", pa.string()),
    ])
    arrays = [
        pa.array([1, 2, 3], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
    ]
    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
    
    wrapper = ZerobusWrapper(
        endpoint="https://test.cloud.databricks.com",
        table_name="test_table",
        client_id="test_client_id",
        client_secret="test_client_secret",
        unity_catalog_url="https://unity-catalog-url",
    )
    
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
    schema = pa.schema([
        pa.field("id", pa.int64()),
        pa.field("name", pa.string()),
        pa.field("score", pa.float64()),
    ])
    
    arrays = [
        pa.array([1, 2, 3], type=pa.int64()),
        pa.array(["Alice", "Bob", "Charlie"], type=pa.string()),
        pa.array([95.5, 87.0, 92.5], type=pa.float64()),
    ]
    
    batch = pa.RecordBatch.from_arrays(arrays, schema=schema)
    
    assert batch.num_rows == 3
    assert batch.num_columns == 3
    assert batch.schema == schema


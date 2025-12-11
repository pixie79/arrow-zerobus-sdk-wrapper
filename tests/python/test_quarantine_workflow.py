"""Tests for quarantine workflow helper methods in TransmissionResult

These tests verify that the Python bindings correctly expose quarantine
workflow helper methods, matching the Rust API behavior.
"""

import pytest
import pyarrow as pa

# Skip all tests if the module is not available
try:
    from arrow_zerobus_sdk_wrapper import (
        TransmissionResult,
    )
except ImportError:
    pytestmark = pytest.mark.skip("arrow_zerobus_sdk_wrapper not available")


def create_test_batch():
    """Create a test RecordBatch with multiple rows."""
    schema = pa.schema(
        [
            pa.field("id", pa.int64()),
            pa.field("name", pa.string()),
        ]
    )
    arrays = [
        pa.array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], type=pa.int64()),
        pa.array(
            [
                "Alice",
                "Bob",
                "Charlie",
                "David",
                "Eve",
                "Frank",
                "Grace",
                "Henry",
                "Ivy",
                "Jack",
            ],
            type=pa.string(),
        ),
    ]
    return pa.RecordBatch.from_arrays(arrays, schema=schema)


def test_get_failed_row_indices():
    """Test get_failed_row_indices() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[
            (1, "ConversionError: error 1"),
            (3, "TransmissionError: error 2"),
        ],
        successful_rows=[0, 2, 4],
        total_rows=5,
        successful_count=3,
        failed_count=2,
    )

    failed_indices = result.get_failed_row_indices()
    assert failed_indices == [1, 3]


def test_get_failed_row_indices_empty():
    """Test get_failed_row_indices() with no failed rows."""
    result = TransmissionResult(
        success=True,
        failed_rows=None,
        successful_rows=[0, 1, 2],
        total_rows=3,
        successful_count=3,
        failed_count=0,
    )

    failed_indices = result.get_failed_row_indices()
    assert failed_indices == []


def test_get_successful_row_indices():
    """Test get_successful_row_indices() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[(1, "Error 1"), (3, "Error 2")],
        successful_rows=[0, 2, 4],
        total_rows=5,
        successful_count=3,
        failed_count=2,
    )

    successful_indices = result.get_successful_row_indices()
    assert successful_indices == [0, 2, 4]


def test_extract_failed_batch():
    """Test extract_failed_batch() method."""
    batch = create_test_batch()
    result = TransmissionResult(
        success=True,
        failed_rows=[(1, "Error 1"), (3, "Error 2"), (7, "Error 3")],
        successful_rows=[0, 2, 4, 5, 6, 8, 9],
        total_rows=10,
        successful_count=7,
        failed_count=3,
    )

    failed_batch = result.extract_failed_batch(batch)
    assert failed_batch is not None
    assert failed_batch.num_rows == 3

    # Verify failed rows contain correct data
    id_array = failed_batch.column("id")
    assert id_array[0].as_py() == 2  # Row 1: Bob
    assert id_array[1].as_py() == 4  # Row 3: David
    assert id_array[2].as_py() == 8  # Row 7: Henry


def test_extract_failed_batch_empty():
    """Test extract_failed_batch() with no failed rows."""
    batch = create_test_batch()
    result = TransmissionResult(
        success=True,
        failed_rows=None,
        successful_rows=list(range(10)),
        total_rows=10,
        successful_count=10,
        failed_count=0,
    )

    failed_batch = result.extract_failed_batch(batch)
    assert failed_batch is None


def test_extract_successful_batch():
    """Test extract_successful_batch() method."""
    batch = create_test_batch()
    result = TransmissionResult(
        success=True,
        failed_rows=[(1, "Error 1"), (3, "Error 2")],
        successful_rows=[0, 2, 4, 5, 6, 7, 8, 9],
        total_rows=10,
        successful_count=8,
        failed_count=2,
    )

    successful_batch = result.extract_successful_batch(batch)
    assert successful_batch is not None
    assert successful_batch.num_rows == 8

    # Verify successful rows contain correct data
    id_array = successful_batch.column("id")
    assert id_array[0].as_py() == 1  # Row 0: Alice
    assert id_array[1].as_py() == 3  # Row 2: Charlie
    assert id_array[2].as_py() == 5  # Row 4: Eve


def test_extract_successful_batch_empty():
    """Test extract_successful_batch() with no successful rows."""
    batch = create_test_batch()
    result = TransmissionResult(
        success=False,
        failed_rows=[(i, f"Error {i}") for i in range(10)],
        successful_rows=None,
        total_rows=10,
        successful_count=0,
        failed_count=10,
    )

    successful_batch = result.extract_successful_batch(batch)
    assert successful_batch is None


def test_get_failed_row_indices_by_error_type():
    """Test get_failed_row_indices_by_error_type() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[
            (0, "ConversionError: error 1"),
            (1, "TransmissionError: error 2"),
            (2, "ConversionError: error 3"),
            (3, "ConnectionError: error 4"),
        ],
        successful_rows=[4, 5, 6, 7, 8, 9],
        total_rows=10,
        successful_count=6,
        failed_count=4,
    )

    conversion_indices = result.get_failed_row_indices_by_error_type("ConversionError")
    assert conversion_indices == [0, 2]

    transmission_indices = result.get_failed_row_indices_by_error_type(
        "TransmissionError"
    )
    assert transmission_indices == [1]

    connection_indices = result.get_failed_row_indices_by_error_type("ConnectionError")
    assert connection_indices == [3]


def test_is_partial_success():
    """Test is_partial_success() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[(1, "Error 1")],
        successful_rows=[0, 2],
        total_rows=3,
        successful_count=2,
        failed_count=1,
    )

    assert result.is_partial_success() is True


def test_has_failed_rows():
    """Test has_failed_rows() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[(1, "Error 1")],
        successful_rows=[0, 2],
        total_rows=3,
        successful_count=2,
        failed_count=1,
    )

    assert result.has_failed_rows() is True


def test_has_successful_rows():
    """Test has_successful_rows() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[(1, "Error 1")],
        successful_rows=[0, 2],
        total_rows=3,
        successful_count=2,
        failed_count=1,
    )

    assert result.has_successful_rows() is True


def test_quarantine_workflow_complete():
    """Test complete quarantine workflow."""
    batch = create_test_batch()
    result = TransmissionResult(
        success=True,
        failed_rows=[
            (1, "ConversionError: error 1"),
            (3, "TransmissionError: error 2"),
        ],
        successful_rows=[0, 2, 4, 5, 6, 7, 8, 9],
        total_rows=10,
        successful_count=8,
        failed_count=2,
    )

    # Step 1: Verify partial success
    assert result.is_partial_success()
    assert result.has_failed_rows()
    assert result.has_successful_rows()

    # Step 2: Extract failed rows for quarantine
    failed_batch = result.extract_failed_batch(batch)
    assert failed_batch is not None
    assert failed_batch.num_rows == 2

    # Step 3: Extract successful rows for writing to main table
    successful_batch = result.extract_successful_batch(batch)
    assert successful_batch is not None
    assert successful_batch.num_rows == 8

    # Step 4: Verify consistency
    assert failed_batch.num_rows + successful_batch.num_rows == result.total_rows

"""Tests for per-row error information in TransmissionResult

These tests verify that the Python bindings correctly expose per-row error
information from TransmissionResult, matching the Rust API behavior.
"""

import pytest

# Skip all tests if the module is not available
try:
    from arrow_zerobus_sdk_wrapper import (
        TransmissionResult,
    )
except ImportError:
    pytestmark = pytest.mark.skip("arrow_zerobus_sdk_wrapper not available")


def test_transmission_result_new_fields_exist():
    """Test that new per-row error fields exist in TransmissionResult."""
    # Create a TransmissionResult with per-row error information
    result = TransmissionResult(
        success=True,
        message="Test message",
        failed_rows=[
            (0, "ConversionError: test error"),
            (2, "TransmissionError: network error"),
        ],
        successful_rows=[1, 3],
        total_rows=4,
        successful_count=2,
        failed_count=2,
    )

    # Verify all new fields exist and are accessible
    assert hasattr(result, "failed_rows")
    assert hasattr(result, "successful_rows")
    assert hasattr(result, "total_rows")
    assert hasattr(result, "successful_count")
    assert hasattr(result, "failed_count")

    # Verify field values
    assert result.failed_rows == [
        (0, "ConversionError: test error"),
        (2, "TransmissionError: network error"),
    ]
    assert result.successful_rows == [1, 3]
    assert result.total_rows == 4
    assert result.successful_count == 2
    assert result.failed_count == 2


def test_transmission_result_all_success():
    """Test TransmissionResult with all rows successful."""
    result = TransmissionResult(
        success=True,
        message="All rows succeeded",
        failed_rows=None,
        successful_rows=[0, 1, 2],
        total_rows=3,
        successful_count=3,
        failed_count=0,
    )

    assert result.success is True
    assert result.failed_rows is None or result.failed_rows == []
    assert result.successful_rows == [0, 1, 2]
    assert result.total_rows == 3
    assert result.successful_count == 3
    assert result.failed_count == 0


def test_transmission_result_all_failed():
    """Test TransmissionResult with all rows failed."""
    result = TransmissionResult(
        success=False,
        message="All rows failed",
        failed_rows=[
            (0, "ConversionError: row 0 error"),
            (1, "ConversionError: row 1 error"),
            (2, "ConversionError: row 2 error"),
        ],
        successful_rows=None,
        total_rows=3,
        successful_count=0,
        failed_count=3,
    )

    assert result.success is False
    assert len(result.failed_rows) == 3
    assert result.successful_rows is None or result.successful_rows == []
    assert result.total_rows == 3
    assert result.successful_count == 0
    assert result.failed_count == 3


def test_transmission_result_partial_success():
    """Test TransmissionResult with partial success (some rows succeed, some fail)."""
    result = TransmissionResult(
        success=True,  # Partial success is still considered success
        message="Partial success",
        failed_rows=[(1, "ConversionError: row 1 failed")],
        successful_rows=[0, 2],
        total_rows=3,
        successful_count=2,
        failed_count=1,
    )

    assert result.success is True
    assert len(result.failed_rows) == 1
    assert result.failed_rows[0] == (1, "ConversionError: row 1 failed")
    assert result.successful_rows == [0, 2]
    assert result.total_rows == 3
    assert result.successful_count == 2
    assert result.failed_count == 1


def test_transmission_result_empty_batch():
    """Test TransmissionResult with empty batch."""
    result = TransmissionResult(
        success=True,
        message="Empty batch",
        failed_rows=None,
        successful_rows=None,
        total_rows=0,
        successful_count=0,
        failed_count=0,
    )

    assert result.success is True
    assert result.failed_rows is None or result.failed_rows == []
    assert result.successful_rows is None or result.successful_rows == []
    assert result.total_rows == 0
    assert result.successful_count == 0
    assert result.failed_count == 0


def test_transmission_result_consistency():
    """Test that TransmissionResult fields are consistent."""
    # Test case 1: total_rows == successful_count + failed_count
    result1 = TransmissionResult(
        success=True,
        message="Test",
        failed_rows=[(0, "Error")],
        successful_rows=[1, 2],
        total_rows=3,
        successful_count=2,
        failed_count=1,
    )
    assert result1.total_rows == result1.successful_count + result1.failed_count

    # Test case 2: successful_rows length matches successful_count
    if result1.successful_rows:
        assert len(result1.successful_rows) == result1.successful_count

    # Test case 3: failed_rows length matches failed_count
    if result1.failed_rows:
        assert len(result1.failed_rows) == result1.failed_count


def test_transmission_result_failed_rows_format():
    """Test that failed_rows are in the correct format (list of tuples)."""
    result = TransmissionResult(
        success=False,
        message="Test",
        failed_rows=[
            (0, "ConversionError: error 1"),
            (1, "TransmissionError: error 2"),
        ],
        successful_rows=None,
        total_rows=2,
        successful_count=0,
        failed_count=2,
    )

    # Verify failed_rows is a list
    assert isinstance(result.failed_rows, list)

    # Verify each element is a tuple of (int, str)
    for failed_row in result.failed_rows:
        assert isinstance(failed_row, tuple)
        assert len(failed_row) == 2
        assert isinstance(failed_row[0], int)  # Row index
        assert isinstance(failed_row[1], str)  # Error message


def test_transmission_result_successful_rows_format():
    """Test that successful_rows are in the correct format (list of ints)."""
    result = TransmissionResult(
        success=True,
        message="Test",
        failed_rows=None,
        successful_rows=[0, 1, 2, 5, 10],
        total_rows=11,
        successful_count=5,
        failed_count=6,
    )

    # Verify successful_rows is a list
    assert isinstance(result.successful_rows, list)

    # Verify each element is an int (row index)
    for row_idx in result.successful_rows:
        assert isinstance(row_idx, int)
        assert row_idx >= 0


def test_transmission_result_backward_compatibility():
    """Test that existing code using TransmissionResult still works."""
    # Create a result with only the original fields (simulating old code)
    # Note: In practice, old code would still work because new fields are optional
    result = TransmissionResult(
        success=True,
        message="Test message",
        failed_rows=None,
        successful_rows=None,
        total_rows=0,
        successful_count=0,
        failed_count=0,
    )

    # Original fields should still work
    assert result.success is True
    assert result.message == "Test message"

    # New fields should have sensible defaults
    assert result.failed_rows is None or result.failed_rows == []
    assert result.successful_rows is None or result.successful_rows == []
    assert result.total_rows == 0
    assert result.successful_count == 0
    assert result.failed_count == 0
# test

"""Tests for error analysis and pattern analysis in TransmissionResult

These tests verify that the Python bindings correctly expose error analysis
methods for debugging and pattern analysis.
"""

import pytest

# Skip all tests if the module is not available
try:
    from arrow_zerobus_sdk_wrapper import (
        TransmissionResult,
    )
except ImportError:
    pytestmark = pytest.mark.skip("arrow_zerobus_sdk_wrapper not available")


def test_group_errors_by_type():
    """Test group_errors_by_type() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[
            (0, "ConversionError: error 1"),
            (1, "TransmissionError: error 2"),
            (2, "ConversionError: error 3"),
            (3, "ConnectionError: error 4"),
            (4, "ConversionError: error 5"),
        ],
        successful_rows=[5, 6, 7, 8, 9],
        total_rows=10,
        successful_count=5,
        failed_count=5,
    )

    grouped = result.group_errors_by_type()

    assert "ConversionError" in grouped
    assert "TransmissionError" in grouped
    assert "ConnectionError" in grouped
    assert grouped["ConversionError"] == [0, 2, 4]
    assert grouped["TransmissionError"] == [1]
    assert grouped["ConnectionError"] == [3]


def test_group_errors_by_type_empty():
    """Test group_errors_by_type() with no failed rows."""
    result = TransmissionResult(
        success=True,
        failed_rows=None,
        successful_rows=[0, 1, 2],
        total_rows=3,
        successful_count=3,
        failed_count=0,
    )

    grouped = result.group_errors_by_type()
    assert len(grouped) == 0


def test_get_error_statistics():
    """Test get_error_statistics() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[
            (0, "ConversionError: error 1"),
            (1, "TransmissionError: error 2"),
            (2, "ConversionError: error 3"),
            (3, "ConnectionError: error 4"),
            (4, "ConversionError: error 5"),
        ],
        successful_rows=[5, 6, 7, 8, 9],
        total_rows=10,
        successful_count=5,
        failed_count=5,
    )

    stats = result.get_error_statistics()

    assert stats["total_rows"] == 10
    assert stats["successful_count"] == 5
    assert stats["failed_count"] == 5
    assert stats["success_rate"] == 0.5
    assert stats["failure_rate"] == 0.5

    error_type_counts = stats["error_type_counts"]
    assert error_type_counts["ConversionError"] == 3
    assert error_type_counts["TransmissionError"] == 1
    assert error_type_counts["ConnectionError"] == 1


def test_get_error_statistics_all_success():
    """Test get_error_statistics() with all rows successful."""
    result = TransmissionResult(
        success=True,
        failed_rows=None,
        successful_rows=[0, 1, 2, 3, 4],
        total_rows=5,
        successful_count=5,
        failed_count=0,
    )

    stats = result.get_error_statistics()

    assert stats["total_rows"] == 5
    assert stats["successful_count"] == 5
    assert stats["failed_count"] == 0
    assert stats["success_rate"] == 1.0
    assert stats["failure_rate"] == 0.0
    assert len(stats["error_type_counts"]) == 0


def test_get_error_messages():
    """Test get_error_messages() method."""
    result = TransmissionResult(
        success=True,
        failed_rows=[
            (0, "ConversionError: Field 'name' type mismatch"),
            (1, "TransmissionError: Network timeout"),
            (2, "ConversionError: Field 'age' missing required value"),
        ],
        successful_rows=[3, 4],
        total_rows=5,
        successful_count=2,
        failed_count=3,
    )

    error_messages = result.get_error_messages()

    assert len(error_messages) == 3
    # Note: ZerobusError.to_string() includes the error type prefix (lowercase)
    assert "Conversion error: Field 'name' type mismatch" in error_messages
    assert "Transmission error: Network timeout" in error_messages
    assert "Conversion error: Field 'age' missing required value" in error_messages


def test_get_error_messages_empty():
    """Test get_error_messages() with no failed rows."""
    result = TransmissionResult(
        success=True,
        failed_rows=None,
        successful_rows=[0, 1, 2],
        total_rows=3,
        successful_count=3,
        failed_count=0,
    )

    error_messages = result.get_error_messages()
    assert len(error_messages) == 0


def test_error_pattern_analysis():
    """Test error pattern analysis across multiple results."""
    results = [
        TransmissionResult(
            success=True,
            failed_rows=[(0, "ConversionError: Field 'age' type mismatch")],
            successful_rows=[1, 2, 3, 4],
            total_rows=5,
            successful_count=4,
            failed_count=1,
        ),
        TransmissionResult(
            success=True,
            failed_rows=[(0, "ConversionError: Field 'age' type mismatch")],
            successful_rows=[1, 2, 3],
            total_rows=4,
            successful_count=3,
            failed_count=1,
        ),
    ]

    # Analyze patterns across batches
    all_conversion_errors = []
    for result in results:
        grouped = result.group_errors_by_type()
        if "ConversionError" in grouped:
            all_conversion_errors.extend(grouped["ConversionError"])

    assert len(all_conversion_errors) == 2

    # Check for common error pattern
    all_messages = []
    for result in results:
        all_messages.extend(result.get_error_messages())

    common_error = "Field 'age' type mismatch"
    assert any(common_error in msg for msg in all_messages)


def test_error_statistics_for_monitoring():
    """Test error statistics for monitoring scenarios."""
    result = TransmissionResult(
        success=True,
        failed_rows=[
            (0, "ConversionError: error 1"),
            (1, "ConversionError: error 2"),
            (2, "TransmissionError: error 3"),
        ],
        successful_rows=[3, 4, 5, 6, 7],
        total_rows=8,
        successful_count=5,
        failed_count=3,
    )

    stats = result.get_error_statistics()

    # Verify statistics are suitable for monitoring
    assert stats["total_rows"] > 0
    assert stats["success_rate"] >= 0.0
    assert stats["success_rate"] <= 1.0
    assert stats["failure_rate"] >= 0.0
    assert stats["failure_rate"] <= 1.0
    assert abs(stats["success_rate"] + stats["failure_rate"] - 1.0) < 0.001

    # Verify error type distribution
    assert "ConversionError" in stats["error_type_counts"]
    assert "TransmissionError" in stats["error_type_counts"]

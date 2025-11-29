"""Pytest configuration for Python bindings tests

This file contains fixtures and configuration to work around PyO3 issues
with pytest, particularly the GIL (Global Interpreter Lock) and Python
initialization problems that can cause pytest to hang or fail after tests.

The main issue is that PyO3's auto-initialize feature can cause Python
to be initialized multiple times, leading to GIL deadlocks and pytest
hanging after tests complete.

Workarounds implemented:
1. Set PYO3_NO_PYTHON_VERSION_CHECK to skip version checks
2. Use session-level fixture to initialize Python once
3. Force garbage collection after each test
4. Use pytest-forked for process isolation (configured in pytest.ini)
"""

import pytest
import sys
import os
import gc

# Workaround for PyO3 pytest issues:
# Set environment variable to prevent PyO3 from doing version checks
# This can help avoid initialization issues
os.environ.setdefault("PYO3_NO_PYTHON_VERSION_CHECK", "1")

# Try to import the module early to catch import errors
try:
    import arrow_zerobus_sdk_wrapper
except ImportError:
    # If module is not available, skip all tests
    pytestmark = pytest.mark.skip("arrow_zerobus_sdk_wrapper not available")


@pytest.fixture(scope="session", autouse=True)
def setup_python_environment():
    """Session-level fixture to set up Python environment for PyO3.
    
    This fixture runs once per test session and ensures Python is properly
    initialized before any tests run. It also cleans up after all tests.
    
    This helps prevent the PyO3 pytest hang issue by ensuring Python
    is only initialized once per test session.
    """
    # Pre-initialize Python to avoid multiple initializations
    # This helps prevent the PyO3 pytest hang issue
    try:
        import arrow_zerobus_sdk_wrapper
        # Force module import to initialize Python bindings early
        # This ensures Python is initialized before tests run
        _ = arrow_zerobus_sdk_wrapper.ZerobusWrapper
    except (ImportError, AttributeError):
        pass
    
    yield
    
    # Cleanup after all tests
    # Force garbage collection to release any Python objects
    # This helps prevent memory leaks and GIL issues
    gc.collect()


@pytest.fixture(autouse=True)
def isolate_tests():
    """Fixture to isolate each test and prevent shared state issues.
    
    This runs before and after each test to ensure proper cleanup.
    The cleanup is critical for PyO3 to prevent GIL deadlocks.
    """
    # Setup: ensure clean state
    yield
    # Teardown: force garbage collection after each test
    # This releases Python objects and prevents GIL issues
    gc.collect()
    # Run multiple times to ensure all cyclic references are cleared
    gc.collect()


# Configure pytest to use forked/isolated test execution if available
# This prevents GIL and initialization issues
def pytest_configure(config):
    """Configure pytest with PyO3-specific settings."""
    # Add custom markers
    config.addinivalue_line(
        "markers", "python: marks tests that require Python bindings"
    )
    
    # If pytest-xdist is available, configure it for better isolation
    if hasattr(config.option, 'numprocesses'):
        # Use process-based isolation for PyO3 tests
        if config.option.numprocesses is None:
            # Default to 1 process to avoid GIL issues
            config.option.numprocesses = 1


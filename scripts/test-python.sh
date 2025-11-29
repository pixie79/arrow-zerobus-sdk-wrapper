#!/bin/bash
# Test script for Python bindings with PyO3 pytest workaround
# This script sets up the environment and runs Python tests with proper isolation

set -e

# Find Python executable
if command -v python3.11 &> /dev/null; then
    PYTHON_EXEC="python3.11"
elif command -v python3 &> /dev/null; then
    PYTHON_EXEC="python3"
else
    echo "Error: Python 3.11+ not found in PATH" >&2
    exit 1
fi

# Verify Python version
PYTHON_VERSION=$($PYTHON_EXEC --version 2>&1 | awk '{print $2}')
PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d. -f1)
PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d. -f2)

if [ "$PYTHON_MAJOR" -lt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -lt 11 ]); then
    echo "Warning: Python version $PYTHON_VERSION is less than 3.11" >&2
    echo "Continuing anyway, but Python bindings may not work correctly" >&2
fi

# Export PYO3_PYTHON for PyO3
export PYO3_PYTHON="$PYTHON_EXEC"

# Set environment variable to work around PyO3 pytest issues
export PYO3_NO_PYTHON_VERSION_CHECK=1

echo "Using Python: $PYTHON_EXEC ($PYTHON_VERSION)"
echo "Setting up Python environment..."

# Check if we're in a virtual environment
if [ -z "$VIRTUAL_ENV" ]; then
    echo "Warning: Not in a virtual environment. Consider using a venv."
fi

# Install/upgrade required packages
echo "Installing Python test dependencies..."
$PYTHON_EXEC -m pip install --upgrade pip --quiet
$PYTHON_EXEC -m pip install pytest pytest-cov pytest-forked --quiet

# Check if maturin is needed to build the extension
if [ ! -f "target/release/libarrow_zerobus_sdk_wrapper.so" ] && [ ! -f "target/release/libarrow_zerobus_sdk_wrapper.dylib" ]; then
    echo "Python extension not found. Building with maturin..."
    if ! command -v maturin &> /dev/null; then
        echo "Installing maturin..."
        $PYTHON_EXEC -m pip install maturin --quiet
    fi
    maturin develop --release
fi

echo "Running Python tests with PyO3 workaround..."
echo "Command: pytest tests/python/ -v --forked $@"

# Run pytest with forked execution to prevent GIL issues
$PYTHON_EXEC -m pytest tests/python/ -v --forked "$@"


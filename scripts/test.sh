#!/bin/bash
# Test script for arrow-zerobus-sdk-wrapper
# Automatically sets PYO3_PYTHON when running tests with python feature

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

echo "Using Python: $PYTHON_EXEC ($PYTHON_VERSION)"
echo "Running: cargo test $@"

# Run cargo test with all arguments passed through
cargo test "$@"


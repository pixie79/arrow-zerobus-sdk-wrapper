#!/bin/bash
# Check test coverage for Rust code
# Target: ≥90% coverage per file

set -e

echo "Running test coverage check..."

# Install cargo-tarpaulin if not present
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Run tests with coverage
echo "Running tests with coverage (Python feature enabled)..."
cargo tarpaulin --features python --out Xml --output-dir coverage/ --timeout 120

# Generate HTML report
cargo tarpaulin --features python --out Html --output-dir coverage/ --timeout 120

# Generate LCOV report
cargo tarpaulin --features python --out Lcov --output-dir coverage/ --timeout 120

echo "Coverage reports generated in coverage/ directory"
echo "HTML report: coverage/tarpaulin-report.html"
echo "XML report: coverage/cobertura.xml"
echo "LCOV report: coverage/lcov.info"

# Check if coverage meets threshold (90%)
COVERAGE=$(cargo tarpaulin --features python --out Stdout --timeout 120 2>&1 | grep -oP '^\d+\.\d+%' | head -1 | sed 's/%//')

if [ -z "$COVERAGE" ]; then
    echo "Warning: Could not extract coverage percentage"
    exit 0
fi

THRESHOLD=90.0
if (( $(echo "$COVERAGE < $THRESHOLD" | bc -l) )); then
    echo "❌ Coverage ${COVERAGE}% is below threshold ${THRESHOLD}%"
    exit 1
else
    echo "✅ Coverage ${COVERAGE}% meets threshold ${THRESHOLD}%"
fi


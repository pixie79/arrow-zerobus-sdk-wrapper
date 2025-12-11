#!/bin/bash
# Install pre-commit hook for version checking

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

HOOK_FILE="$REPO_ROOT/.git/hooks/pre-commit"
HOOK_CONTENT="$REPO_ROOT/.git/hooks/pre-commit"

# Check if .git/hooks directory exists
if [ ! -d "$REPO_ROOT/.git/hooks" ]; then
    echo "Error: .git/hooks directory not found. Are you in a git repository?"
    exit 1
fi

# Copy pre-commit hook
if [ -f "$REPO_ROOT/.git/hooks/pre-commit" ]; then
    echo "⚠️  Pre-commit hook already exists. Backing up to pre-commit.backup"
    cp "$REPO_ROOT/.git/hooks/pre-commit" "$REPO_ROOT/.git/hooks/pre-commit.backup"
fi

# Create pre-commit hook
cat > "$HOOK_FILE" << 'EOF'
#!/bin/bash
# Pre-commit hook to check version consistency, formatting, linting, and run tests
# This hook runs before each commit to ensure code quality

# Get the repository root
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Track if any checks fail
FAILED=0

# 1. Run version check script
echo "🔍 Checking version consistency..."
if [ -f "$REPO_ROOT/scripts/check_version.sh" ]; then
    "$REPO_ROOT/scripts/check_version.sh"
    if [ $? -ne 0 ]; then
        echo ""
        echo "❌ Pre-commit hook failed: Version mismatch detected"
        echo "   Please ensure versions match in:"
        echo "   - Cargo.toml"
        echo "   - pyproject.toml"
        echo "   - CHANGELOG.md (latest release)"
        FAILED=1
    fi
fi

# 2. Check Rust formatting
echo ""
echo "🔍 Checking Rust formatting..."
if command -v cargo &> /dev/null; then
    if ! cargo fmt --check --all > /dev/null 2>&1; then
        echo "❌ Rust formatting check failed"
        echo "   Run: cargo fmt --all"
        FAILED=1
    fi
else
    echo "⚠️  cargo not found, skipping Rust formatting check"
fi

# 3. Check Python formatting (if Python files changed)
echo ""
echo "🔍 Checking Python formatting..."
if git diff --cached --name-only | grep -qE '\.(py)$'; then
    if command -v python3 &> /dev/null; then
        # Check if black is available
        if python3 -m black --version > /dev/null 2>&1; then
            if ! python3 -m black --check tests/python/ scripts/*.py 2>/dev/null; then
                echo "❌ Python formatting check failed"
                echo "   Run: python3 -m black tests/python/ scripts/*.py"
                FAILED=1
            fi
        else
            echo "⚠️  black not installed, skipping Python formatting check"
        fi
    else
        echo "⚠️  python3 not found, skipping Python formatting check"
    fi
else
    echo "ℹ️  No Python files changed, skipping Python formatting check"
fi

# 4. Run Python tests (if Python files changed or if bindings are available)
echo ""
echo "🔍 Running Python tests..."
if git diff --cached --name-only | grep -qE '\.(py|rs)$|tests/python/|src/python/'; then
    # Check if Python bindings are available
    if python3 -c "import arrow_zerobus_sdk_wrapper" 2>/dev/null; then
        # Set up environment variables for PyO3
        export PYO3_NO_PYTHON_VERSION_CHECK=1
        if command -v python3 &> /dev/null; then
            PYTHON_EXEC=$(python3 -c "import sys; print(sys.executable)")
            export PYO3_PYTHON="$PYTHON_EXEC"
        fi
        
        # Check if pytest is available
        if python3 -m pytest --version > /dev/null 2>&1; then
            # Run tests without coverage for speed (coverage is checked in CI)
            # Use -o to override pytest.ini addopts, skip forked mode for faster execution
            if ! python3 -m pytest tests/python/ -v -o addopts="-v" --no-cov 2>&1; then
                echo "❌ Python tests failed"
                echo "   Run: python3 -m pytest tests/python/ -v"
                FAILED=1
            fi
        else
            echo "⚠️  pytest not installed, skipping Python tests"
            echo "   Install with: pip install pytest pytest-asyncio"
        fi
    else
        echo "ℹ️  Python bindings not available, skipping Python tests"
        echo "   Build with: maturin develop --release"
    fi
else
    echo "ℹ️  No Python/Rust files changed, skipping Python tests"
fi

# Exit with error if any checks failed
if [ $FAILED -eq 1 ]; then
    echo ""
    echo "❌ Pre-commit hook failed. Please fix the issues above before committing."
    exit 1
fi

echo ""
echo "✅ Pre-commit checks passed!"
exit 0
EOF

chmod +x "$HOOK_FILE"
echo "✅ Pre-commit hook installed successfully!"
echo "   The hook will check:"
echo "   - Version consistency (Cargo.toml, pyproject.toml, CHANGELOG.md)"
echo "   - Rust formatting (cargo fmt)"
echo "   - Python formatting (black)"
echo "   - Python tests (if bindings available and files changed)"


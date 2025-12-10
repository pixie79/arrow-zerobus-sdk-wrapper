# Testing Guide

## Running Tests

### Quick Test Run
```bash
cargo test --all-targets
```

### Full Test Suite with Output
```bash
./scripts/run_tests.sh
```

This script:
1. **Builds Rust project** (`cargo build --release`) - Required for Python bindings
2. **Runs all Rust tests** (`cargo test --all-targets`)
3. **Runs Python tests** (only if Rust build and tests pass, and module is importable)
4. Saves output to `testResults/` directory with timestamps
5. Creates a `latest.txt` symlink to the most recent results

**Note**: Python tests are only run if:
- Rust build succeeds
- Rust tests pass
- Python module is importable (requires `maturin develop --release`)

### Test Results

Test results are saved to `testResults/` directory:
- `test_results_YYYYMMDD_HHMMSS.txt` - Full test output
- `test_summary_YYYYMMDD_HHMMSS.txt` - Test summary
- `latest.txt` - Symlink to most recent results

**Note**: The `testResults/` directory is excluded from git (see `.gitignore`)

## Pre-commit Hook

The pre-commit hook automatically runs before each commit:

1. **Code Formatting** (`cargo fmt --check`)
2. **Clippy Linting** (`cargo clippy --all-targets`)
3. **Test Build** (`cargo build --tests`)
4. **Test Execution** (`cargo test --all-targets`)

If any check fails, the commit is blocked. Fix the issues and try again.

### Bypassing Pre-commit Hook

If you need to bypass the pre-commit hook (not recommended):

```bash
git commit --no-verify -m "your message"
```

## Test Coverage

### Running Coverage Analysis

```bash
cargo tarpaulin --all-targets --out Html
```

This generates an HTML coverage report in `tarpaulin-report.html`.

### Coverage Target

- **Target**: ≥90% coverage per file (as per project constitution)
- **Current**: See `TEST_PLAN.md` for detailed coverage analysis

## Test Organization

### Unit Tests
- Location: `tests/unit/`
- Organized by module (wrapper, config, etc.)
- Fast, isolated tests

### Integration Tests
- Location: `tests/integration/`
- Test component interactions
- May require external dependencies

### Contract Tests
- Location: `tests/contract/`
- Verify API contracts
- Ensure backward compatibility

### Python Tests
- Location: `tests/python/`
- Test Python bindings
- Require PyArrow and pytest
- **Prerequisites**: 
  - Rust project must build successfully (`cargo build --release`)
  - Rust tests must pass
  - Python module must be importable (built with `maturin develop --release`)
- **Note**: Python tests are skipped if Rust build/tests fail or if the module isn't importable

### Performance Tests
- Location: `tests/performance/`
- Marked with `#[ignore]` - run manually
- Test large batches, high throughput, etc.

## Running Specific Tests

### Run a specific test
```bash
cargo test test_name
```

### Run tests in a specific module
```bash
cargo test --test test_file_name
```

### Run only unit tests
```bash
cargo test --lib
```

### Run only integration tests
```bash
cargo test --test '*'
```

### Run ignored tests
```bash
cargo test -- --ignored
```

## Continuous Integration

The pre-commit hook ensures:
- All code is properly formatted
- No clippy warnings
- All tests pass

This helps prevent broken code from being committed.

## Python Tests Setup

Python tests require the Rust module to be built and importable. To set up:

```bash
# 1. Install maturin (if not already installed)
pip install maturin

# 2. Build Python bindings in development mode
maturin develop --release

# 3. Verify module is importable
python3 -c "import arrow_zerobus_sdk_wrapper; print('✅ Module importable')"

# 4. Run tests
./scripts/run_tests.sh
```

**Note**: The test runner automatically checks if the module is importable before running Python tests. If it's not available, Python tests are skipped (not a failure).

## Troubleshooting

### Tests fail in pre-commit but pass locally
- Check for uncommitted formatting changes: `cargo fmt --all`
- Check for clippy warnings: `cargo clippy --all-targets`
- Ensure all tests pass: `cargo test --all-targets`

### Python tests skipped
- **Module not importable**: Build with `maturin develop --release`
- **Rust build failed**: Fix Rust build errors first
- **Rust tests failed**: Fix Rust test failures first
- Python tests only run if Rust build and tests pass

### Test output not saving
- Check that `testResults/` directory exists and is writable
- Verify script has execute permissions: `chmod +x scripts/run_tests.sh`

### Pre-commit hook not running
- Verify hook is executable: `chmod +x .git/hooks/pre-commit`
- Check hook exists: `ls -la .git/hooks/pre-commit`


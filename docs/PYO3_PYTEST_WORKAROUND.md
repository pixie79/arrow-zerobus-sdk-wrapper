# PyO3 Pytest Workaround

## Problem

PyO3's `auto-initialize` feature can cause pytest to hang or fail after tests complete. This is due to:

1. **Multiple Python Initializations**: PyO3 may try to initialize Python multiple times, causing conflicts
2. **GIL (Global Interpreter Lock) Issues**: The GIL can become deadlocked when Python is initialized multiple times
3. **Test Isolation**: Tests may share Python state, causing unpredictable behavior

## Solution

We've implemented a multi-layered workaround:

### 1. pytest-forked for Process Isolation

The `--forked` flag runs each test in a separate process, ensuring complete isolation:

```bash
pytest tests/python/ -v --forked
```

This prevents GIL deadlocks by ensuring each test has its own Python interpreter.

### 2. conftest.py Configuration

The `tests/python/conftest.py` file includes:

- **Session-level fixture**: Initializes Python once per test session
- **Test-level fixture**: Forces garbage collection after each test
- **Environment variables**: Sets `PYO3_NO_PYTHON_VERSION_CHECK=1` to skip version checks

### 3. Environment Variables

Set these environment variables before running tests:

```bash
export PYO3_NO_PYTHON_VERSION_CHECK=1
export PYO3_PYTHON=/path/to/python3
```

### 4. pytest.ini Configuration

The `pytest.ini` file is configured with:

```ini
addopts = 
    --forked
    # ... other options
```

## Usage

### Recommended: Use the Helper Script

```bash
./scripts/test-python.sh
```

This script:
- Detects Python 3.11+
- Sets up the environment
- Installs dependencies
- Builds the extension if needed
- Runs tests with proper isolation

### Manual Setup

```bash
# 1. Install dependencies
pip install pytest pytest-cov pytest-forked

# 2. Build Python extension
maturin develop --release

# 3. Set environment variables
export PYO3_NO_PYTHON_VERSION_CHECK=1
export PYO3_PYTHON=$(which python3)

# 4. Run tests
pytest tests/python/ -v --forked
```

## CI/CD Integration

The `.github/workflows/ci.yml` workflow includes:

```yaml
- name: Install pytest and dependencies
  run: |
    pip install pytest pytest-cov pytest-forked

- name: Run Python tests
  run: |
    export PYO3_NO_PYTHON_VERSION_CHECK=1
    pytest tests/python/ -v --forked
```

## Troubleshooting

### Tests Still Hanging

1. **Check pytest-forked is installed**:
   ```bash
   pip list | grep pytest-forked
   ```

2. **Verify --forked flag is used**:
   ```bash
   pytest tests/python/ -v --forked
   ```

3. **Check Python version**:
   ```bash
   python3 --version  # Should be 3.11+
   ```

4. **Verify environment variables**:
   ```bash
   echo $PYO3_NO_PYTHON_VERSION_CHECK
   echo $PYO3_PYTHON
   ```

### Import Errors

If you see `ImportError: No module named 'arrow_zerobus_sdk_wrapper'`:

1. **Build the extension**:
   ```bash
   maturin develop --release
   ```

2. **Verify installation**:
   ```bash
   python3 -c "import arrow_zerobus_sdk_wrapper; print('OK')"
   ```

### GIL Deadlocks

If tests still deadlock:

1. **Increase test isolation**:
   ```bash
   pytest tests/python/ -v --forked --forked-subprocess
   ```

2. **Run tests sequentially**:
   ```bash
   pytest tests/python/ -v --forked -n 1
   ```

## References

- [PyO3 Documentation](https://pyo3.rs/)
- [pytest-forked Documentation](https://pytest-forked.readthedocs.io/)
- [PyO3 GitHub Issues](https://github.com/PyO3/pyo3/issues)


# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2025-12-11

### Added
- **feat**: Automatic failure rate backoff - Client-side backoff mechanism with jitter that triggers when network/transmission failure rate exceeds 1%
- **feat**: Per-table failure rate tracking - Failure rate statistics tracked independently per table with sliding window (5 minutes)
- **feat**: Batched future collection - Improved throughput by collecting multiple `ingest_record()` futures and awaiting them in batches (1000 records or 10MB)
- **feat**: Arrow IPC Stream format support - Switched debug Arrow files to `.arrows` format (Arrow IPC Stream) for better compatibility with tools like DuckDB
- **feat**: Record-based file rotation - Both Arrow and Protobuf debug files now rotate every 1000 records (configurable via `ROTATION_BATCH_SIZE`)
- **tests**: Comprehensive test coverage for failure rate backoff feature (integration tests covering triggering, blocking, recovery, isolation, error counting)

### Changed
- **enhancement**: Improved batch transmission performance - Records are now queued concurrently and flushed periodically instead of sequentially
- **enhancement**: Enhanced public `flush()` method - Now includes explicit Zerobus stream flushing to ensure buffered records are transmitted
- **enhancement**: Better file handle management - Proper file closing and rotation for debug files with mutable path tracking

### Fixed
- **fix**: Critical data loss issue - Added mandatory `stream.flush()` calls after batch transmission to ensure buffered records are sent to Databricks (resolves "thousands of records dropping" issue)
- **fix**: Debug Arrow files not readable by DuckDB - Switched from Arrow IPC File format to Arrow IPC Stream format (`.arrows` extension) with proper file finalization (resolves #11)
- **fix**: Debug file rotation - Both Arrow and Protobuf debug files now properly rotate based on record count with correct file closing

---

## [0.6.0] - 2025-12-11

### Added
- **feat**: Per-row error tracking - `TransmissionResult` now includes per-row error information enabling partial batch success handling
- **feat**: Per-row error fields - Added `failed_rows`, `successful_rows`, `total_rows`, `successful_count`, `failed_count` to `TransmissionResult`
- **feat**: Quarantine workflow helpers - Methods to extract failed/successful batches: `extract_failed_batch()`, `extract_successful_batch()`
- **feat**: Error analysis helpers - `group_errors_by_type()`, `get_error_statistics()`, `get_error_messages()` for pattern analysis
- **feat**: Error type filtering - `get_failed_row_indices_by_error_type()` to filter failed rows by error type
- **feat**: Partial batch success - Support for batches where some rows succeed and others fail
- **feat**: Enhanced error messages - Error messages now include field names, row indices, and detailed context
- **feat**: Python bindings for per-row errors - All new methods available in Python API
- **feat**: `ErrorStatistics` struct - Comprehensive error statistics with success/failure rates and error type counts
- **tests**: Comprehensive test suite for per-row error handling (unit, integration, contract tests)
- **docs**: Updated quickstart guide with per-row error examples and patterns
- **docs**: Enhanced rustdoc documentation with examples and edge case descriptions

### Changed
- **breaking**: `record_batch_to_protobuf_bytes()` now returns `ProtobufConversionResult` instead of `Result<Vec<Vec<u8>>, ZerobusError>` to support per-row error collection
- **enhancement**: `TransmissionResult` now tracks per-row success/failure instead of batch-level only
- **enhancement**: Error messages include more context (field names, row indices) for better debugging
- **chore**: Updated `otlp-arrow-library` dependency to tag `v0.6.0`

### Fixed
- **fix**: Empty batch handling - Properly handles batches with 0 rows
- **fix**: Batch-level vs per-row error distinction - Clear separation between batch-level errors (auth, connection) and per-row errors

---

## [0.5.0] - 2025-12-11

### Added
- **feat**: Zerobus writer disabled mode - Add `zerobus_writer_disabled` configuration option to disable Zerobus SDK transmission while maintaining debug file output
- **feat**: Configuration validation for writer disabled mode - Requires `debug_enabled` to be true when `zerobus_writer_disabled` is true
- **feat**: Optional credentials when writer disabled - Credentials become optional when `zerobus_writer_disabled` is true, enabling local development without Databricks access
- **feat**: Early return in batch sending - When writer disabled, `send_batch()` skips all SDK calls and returns success after debug file writing
- **feat**: Python bindings support for writer disabled mode - `zerobus_writer_disabled` parameter added to `PyWrapperConfiguration`
- **tests**: Unit tests for configuration validation with writer disabled mode (5 tests)
- **tests**: Integration tests for debug file writing when writer disabled
- **tests**: Integration tests for success return when writer disabled
- **tests**: Integration tests for network verification (no network calls when disabled)
- **tests**: Integration tests for quickstart validation
- **tests**: Performance benchmarks for writer disabled mode
- **docs**: Comprehensive documentation for writer disabled mode in README.md
- **docs**: Quickstart guide with Rust and Python examples
- **docs**: Coverage verification reports

### Changed
- **chore**: Updated GitHub Actions CI workflow to verify formatting and linting for all languages
- **chore**: Enhanced test coverage verification with cargo-tarpaulin
- **chore**: Improved Python code formatting and linting compliance

### Fixed
- **fix**: Python formatting issues in examples and test files
- **fix**: Python linting issues (unused variables, f-string formatting)

---

## [0.4.0] - 2025-12-10

### Added
- **feat**: Python error class constructors - all error types can now be instantiated from Python with messages
- **feat**: Configuration attribute getters - `WrapperConfiguration` now exposes all fields as Python properties
- **feat**: Comprehensive formatting and linting checks for both Rust and Python
- **feat**: Enhanced GitHub Actions CI workflow with sequential job execution
- **feat**: Automated release tag creation on merge to main/master
- **feat**: Python formatting support with `black`
- **feat**: Python linting support with `ruff`
- **docs**: Updated CI workflow documentation

### Changed
- **chore**: Reorganized GitHub Actions workflow to run formatting/linting → build → tests → release in order
- **chore**: Updated CI to check formatting and linting for all languages before running tests
- **chore**: Improved Python bindings error handling and exception construction
- **chore**: Standardized code formatting across Rust and Python codebases
- **chore**: Removed completed planning documents (PYTHON_BINDINGS_FIX_PLAN.md, TODO.md, TEST_PLAN.md)

### Fixed
- **fix**: Python error classes now support direct instantiation (e.g., `ConfigurationError("message")`)
- **fix**: `WrapperConfiguration` attributes are now accessible from Python (e.g., `config.endpoint`, `config.table_name`)
- **fix**: Rust clippy warnings (redundant closures, unused code annotations)
- **fix**: Python linting issues (unused imports, unused variables)
- **fix**: Rust formatting issues in Python bindings
- **fix**: Python formatting issues in test files

---

## [0.2.0] - 2025-11-29

### Added
- **feat**: New `OtlpSdkConfig` structure aligned with otlp-rust-service SDK requirements
- **feat**: Direct SDK ConfigBuilder usage, eliminating conversion layer
- **feat**: Python test support with PyO3 pytest workaround (`pytest-forked`)
- **feat**: Comprehensive test suite for SDK integration (unit and integration tests)
- **docs**: PyO3 pytest workaround documentation (`docs/PYO3_PYTEST_WORKAROUND.md`)
- **docs**: Python test helper script (`scripts/test-python.sh`)

### Changed
- **BREAKING**: `OtlpConfig` replaced with `OtlpSdkConfig` (breaking change)
  - Removed `extra: HashMap<String, Value>` field
  - Added `output_dir: Option<PathBuf>` field
  - Added `write_interval_secs: u64` field (default: 5)
  - Direct mapping to SDK ConfigBuilder requirements
- **BREAKING**: `ObservabilityManager::new_async()` now accepts `Option<OtlpSdkConfig>` instead of `Option<OtlpConfig>`
- **BREAKING**: Removed synchronous `ObservabilityManager::new()` method (dead code)
- **chore**: Removed ~135 lines of dead code:
  - `create_batch_metrics()` method
  - `create_span_data()` method
  - `convert_config()` method
  - Synchronous `new()` method
- **chore**: Updated observability to use tracing infrastructure (SDK picks up events automatically)
- **chore**: Updated all tests to use `OtlpSdkConfig`
- **chore**: Updated Python bindings to use `OtlpSdkConfig`
- **chore**: Updated configuration loader to support `OtlpSdkConfig`
- **chore**: Updated CI workflow to install `pytest-forked` for Python tests

### Fixed
- **fix**: Python tests now work correctly with PyO3 pytest workaround
- **fix**: SDK initialization uses direct ConfigBuilder instead of conversion layer
- **fix**: Metrics and traces now use SDK infrastructure via tracing events

### Migration Guide

**Before (0.1.x)**:
```rust
use arrow_zerobus_sdk_wrapper::{OtlpConfig, ObservabilityManager};

let config = OtlpConfig {
    endpoint: Some("https://otlp-endpoint".to_string()),
    log_level: "info".to_string(),
    extra: HashMap::new(),
};
```

**After (0.2.0)**:
```rust
use arrow_zerobus_sdk_wrapper::{OtlpSdkConfig, ObservabilityManager};
use std::path::PathBuf;

let config = OtlpSdkConfig {
    endpoint: Some("https://otlp-endpoint".to_string()),
    output_dir: Some(PathBuf::from("/tmp/otlp")),
    write_interval_secs: 5,
    log_level: "info".to_string(),
};
```

---

## [0.1.1] - 2025-11-24

### Changed
- **chore**: Updated `databricks-zerobus-ingest-sdk` dependency to use official release v0.1.1 from GitHub instead of local path dependency
- **chore**: Updated `otlp-arrow-library` dependency to explicitly reference main branch from GitHub
- **chore**: Updated code to be compatible with zerobus-sdk-rs v0.1.1 API changes

### Fixed
- **fix**: Updated `TableProperties` usage to match v0.1.1 API (removed `file_descriptor_set`, `descriptor_proto` is now required)
- **fix**: Updated `ensure_stream` function to accept `DescriptorProto` directly instead of `FileDescriptorProto`
- **fix**: Fixed unused import warnings in observability module

### Dependencies
- **deps**: `databricks-zerobus-ingest-sdk` v0.1.1 (from `https://github.com/databricks/zerobus-sdk-rs.git` tag `v0.1.1`)
- **deps**: `otlp-arrow-library` main branch (from `https://github.com/pixie79/otlp-rust-service.git`)

---

## [0.1.0] - 2025-01-01

### Added

#### Core Features
- **feat**: Initial implementation of Arrow Zerobus SDK Wrapper
- **feat**: Rust SDK API for sending Arrow RecordBatch data to Zerobus
- **feat**: Python 3.11+ bindings via PyO3 with zero-copy data transfer
- **feat**: Automatic Arrow RecordBatch to Protobuf conversion
- **feat**: Thread-safe concurrent operations support

#### Authentication & Retry
- **feat**: OAuth2 client credentials authentication
- **feat**: Automatic token refresh for long-running operations
- **feat**: Exponential backoff with jitter retry strategy
- **feat**: Configurable retry attempts, base delay, and max delay

#### Observability
- **feat**: OpenTelemetry metrics integration via otlp-arrow-library
- **feat**: OpenTelemetry trace collection
- **feat**: Observability configuration in WrapperConfiguration
- **feat**: Metrics for batch size, success/failure rates, and latency
- **feat**: Trace spans for batch transmission operations

#### Debug Output
- **feat**: Optional Arrow IPC file output for debugging
- **feat**: Optional Protobuf file output for debugging
- **feat**: Configurable file rotation based on size limits
- **feat**: Periodic file flushing (configurable interval, default 5 seconds)
- **feat**: Debug file output to `{OUTPUT_DIR}/zerobus/arrow/table.arrow` and `{OUTPUT_DIR}/zerobus/proto/table.proto`

#### Configuration
- **feat**: WrapperConfiguration with builder pattern
- **feat**: YAML and environment variable configuration loading
- **feat**: Debug output configuration (enabled/disabled, output directory, flush interval, max file size)
- **feat**: Retry configuration (max attempts, base delay, max delay)
- **feat**: Observability configuration (enabled/disabled, OTLP endpoint)

#### Python Bindings
- **feat**: PyO3 bindings with Pythonic API design
- **feat**: Async context manager support (`async with`)
- **feat**: Zero-copy PyArrow RecordBatch conversion using IPC serialization
- **feat**: Python exception mapping from Rust errors
- **feat**: Python configuration API matching Rust API

#### Testing
- **feat**: Comprehensive unit tests (target ≥90% coverage per file)
- **feat**: Integration tests for Rust API
- **feat**: Contract tests for API compliance
- **feat**: Python integration tests
- **feat**: Test coverage verification scripts (cargo-tarpaulin, pytest-cov)

#### Documentation
- **feat**: README with Rust and Python examples
- **feat**: Quickstart guide for both Rust and Python
- **feat**: DuckDB integration guide for reading debug files
- **feat**: Rust usage example (`examples/rust_example.rs`)
- **feat**: Python usage example (`examples/python_example.py`)

#### Performance
- **feat**: Performance benchmark infrastructure (latency and throughput)
- **feat**: Benchmarks for different batch sizes (1MB, 5MB, 10MB)

#### Utilities
- **feat**: File rotation utility with timestamp-based naming
- **feat**: Arrow IPC file writer with schema support
- **feat**: Protobuf file writer with newline separators

### Changed

- **chore**: Project structure organized by phases (Setup, Foundational, User Stories, Polish)
- **chore**: Code formatting with rustfmt
- **chore**: Linting with clippy

### Fixed

- **fix**: Arrow to Protobuf conversion for various data types
- **fix**: Token refresh error handling
- **fix**: Retry logic with proper error classification
- **fix**: Python bindings error translation
- **fix**: Observability initialization with async support

### Security

- **security**: Thread-safe operations using Arc<Mutex> for shared state
- **security**: Secure credential handling (no logging of secrets)
- **security**: TLS support via rustls for gRPC connections

### Documentation

- **docs**: API documentation with rustdoc
- **docs**: Python API documentation
- **docs**: DuckDB usage examples for Arrow and Protobuf files
- **docs**: Performance targets and benchmarks

---

[0.4.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.4.0
[0.3.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.3.0
[0.2.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.2.0
[0.1.1]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.1.1
[0.1.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.1.0

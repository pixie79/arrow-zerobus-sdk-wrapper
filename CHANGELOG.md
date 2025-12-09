# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

---

## [0.3.0] - 2025-12-09

### Added
- Performance benchmarks with mock SDK integration
- Cross-platform testing (Linux, macOS, Windows)
- Security review and hardening
- cap-gl-consumer-rust compatibility testing

### Changed
- Code cleanup and refactoring (ongoing)
- **docs**: Amended constitution to v1.1.0 - Added Principle VI: Commit Workflow Standards
  - Requires CHANGELOG.md updates, cargo fmt, cargo clippy, and tests passing before commits
  - Requires all commits to be GPG signed
  - Updated Quality Gates to enforce commit workflow requirements

### Fixed
- **fix**: Updated conversion logic for nested rows to properly handle complex nested message structures
- **fix**: Improved Arrow to Protobuf conversion for nested data types
- **fix**: Enhanced debug output for nested message fields
- **fix**: Updated protobuf serialization handling for nested structures
- **fix**: Fixed CI cargo-tarpaulin installation error by adding --force flag to allow overwriting existing installation

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

## [0.1.0] - 2025-01-XX

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

[0.3.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.3.0
[0.2.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.2.0
[0.1.1]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.1.1
[0.1.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.1.0

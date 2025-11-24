# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [0.1.1] - 2025-11-25

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

## [Unreleased]

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
- **fix**: Fixed CI cargo-tarpaulin installation error by adding --force flag to allow overwriting existing installation

---

[0.1.1]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.1.1
[0.1.0]: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/releases/tag/v0.1.0


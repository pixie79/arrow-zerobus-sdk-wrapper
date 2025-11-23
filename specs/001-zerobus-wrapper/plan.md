# Implementation Plan: Zerobus SDK Wrapper

**Branch**: `001-zerobus-wrapper` | **Date**: 2025-11-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-zerobus-wrapper/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Create a cross-platform Rust SDK wrapper for the Databricks Zerobus Rust SDK that provides a unified API for sending Arrow RecordBatch data to Zerobus. The wrapper will handle protocol conversion (Arrow to Protobuf), authentication, retry logic with exponential backoff and jitter, and automatic token refresh. It will support both Rust and Python applications through PyO3 bindings, integrate OpenTelemetry observability via otlp-rust-service, and provide optional debug file output for development. The implementation will reuse code patterns from cap-gl-consumer-rust to enable future drop-in replacement.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021), Python 3.11+ for bindings

**Primary Dependencies**:
- `databricks-zerobus-ingest-sdk` - Official Databricks Zerobus SDK for gRPC streaming
- `arrow` / `arrow-array` (v57) - Arrow RecordBatch handling
- `prost` / `prost-types` (v0.13) - Protobuf serialization (must match SDK versions)
- `tonic` (v0.10) - gRPC support (must match SDK versions)
- `pyo3` (v0.20) - Python bindings with auto-initialize and extension-module features
- `tokio` (v1.35) - Async runtime with full features
- `opentelemetry` / `opentelemetry_sdk` (v0.31) - OpenTelemetry integration
- `otlp-rust-service` (local path) - OpenTelemetry functionality via library integration
- `serde` / `serde_json` / `serde_yaml` - Serialization and configuration
- `anyhow` / `thiserror` - Error handling
- `tracing` / `tracing-subscriber` - Logging
- `chrono` - Time handling
- `rustls` (v0.23) - TLS support for gRPC

**Storage**: Local file system for debug output (optional, disabled by default)

**Testing**: 
- Rust: `cargo test` with native testing framework
- Python: `pytest` for Python bindings
- Coverage: `cargo-tarpaulin` for Rust (≥90% per file requirement)
- Performance: `criterion` for Rust benchmarks

**Target Platform**: Cross-platform (Linux, macOS, Windows)

**Project Type**: Single library project (Rust SDK with Python bindings)

**Performance Goals**:
- p95 latency under 150ms for batches up to 10MB
- 99.999% success rate under normal network conditions
- Zero performance impact when debug files disabled
- Thread-safe concurrent operations

**Constraints**:
- Must maintain ≥90% test coverage per file (constitution requirement)
- Must be thread-safe for concurrent access
- Must support automatic retry with exponential backoff + jitter (max 5 retries default)
- Must automatically refresh authentication tokens
- Debug file flushing every 5 seconds minimum (configurable)
- Must reuse code patterns from cap-gl-consumer-rust for drop-in replacement compatibility

**Scale/Scope**:
- Single library crate with Python bindings
- Drop-in replacement for zerobus functionality in cap-gl-consumer-rust
- Support for batches of any size (no rejection)
- Cross-platform operation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Code Quality Standards ✓
- ✅ Rust best practices and idiomatic patterns
- ✅ Comprehensive documentation (rustdoc)
- ✅ Proper error handling
- ✅ No unsafe code unless justified

### Testing Standards ✓
- ✅ ≥90% test coverage per file (mandatory)
- ✅ TDD workflow (tests first, then implementation)
- ✅ Unit, integration, and contract tests required
- ✅ Performance tests for critical paths

### User Experience Consistency ✓
- ✅ Consistent API across Rust and Python bindings
- ✅ Synchronized documentation
- ✅ Clear, actionable error messages
- ✅ Feature parity between interfaces

### Performance Requirements ✓
- ✅ p95 latency under 150ms (specified)
- ✅ 99.999% success rate (specified)
- ✅ Bounded memory usage
- ✅ Performance benchmarks required

### Multi-Language Support Architecture ✓
- ✅ Rust core implementation
- ✅ PyO3 bindings for Python
- ✅ Zero-copy data transfer where possible
- ✅ Shared underlying implementation

**Status**: All constitution gates pass. No violations detected.

### Post-Design Constitution Check

After Phase 1 design completion:

**Code Quality Standards** ✓
- ✅ Project structure follows Rust best practices with clear module separation
- ✅ Error handling via thiserror/anyhow for clear error types
- ✅ Documentation requirements specified in API contracts

**Testing Standards** ✓
- ✅ Test structure defined (unit, integration, contract, performance)
- ✅ Coverage requirement (≥90%) documented in constitution
- ✅ TDD workflow will be enforced during implementation

**User Experience Consistency** ✓
- ✅ API contracts defined for both Rust and Python with feature parity
- ✅ Consistent error types across both interfaces
- ✅ Quickstart guide provides examples for both languages

**Performance Requirements** ✓
- ✅ Performance targets specified (p95 < 150ms, 99.999% success rate)
- ✅ Benchmark structure defined (criterion for Rust)
- ✅ Memory usage bounded by design (Arc for sharing, no unbounded buffers)

**Multi-Language Support Architecture** ✓
- ✅ Rust core implementation with PyO3 bindings
- ✅ Shared implementation ensures consistency
- ✅ Zero-copy data transfer via PyArrow integration

**Status**: All constitution gates continue to pass after design phase. No violations detected.

## Project Structure

### Documentation (this feature)

```text
specs/001-zerobus-wrapper/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── rust-api.md      # Rust SDK API contract
│   └── python-api.md    # Python bindings API contract
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── lib.rs                # Library entry point
├── config/
│   ├── mod.rs           # Configuration module
│   ├── types.rs         # Configuration types and validation
│   └── loader.rs         # Configuration loading (YAML, env vars)
├── wrapper/
│   ├── mod.rs           # Main wrapper implementation
│   ├── zerobus.rs       # Zerobus SDK integration
│   ├── conversion.rs    # Arrow to Protobuf conversion
│   ├── retry.rs          # Retry logic with exponential backoff + jitter
│   ├── auth.rs           # Authentication and token refresh
│   └── debug.rs          # Debug file writing (Arrow + Protobuf)
├── observability/
│   ├── mod.rs           # OpenTelemetry integration
│   └── otlp.rs          # Integration with otlp-rust-service
├── error.rs             # Error types and handling
├── python/
│   ├── mod.rs           # Python bindings module
│   └── bindings.rs      # PyO3 bindings implementation
└── utils/
    ├── mod.rs
    └── file_rotation.rs  # Debug file rotation logic

tests/
├── unit/
│   ├── wrapper/
│   ├── conversion/
│   ├── retry/
│   ├── auth/
│   └── debug/
├── integration/
│   ├── test_rust_api.rs
│   ├── test_python_bindings.rs
│   ├── test_zerobus_integration.rs
│   └── test_observability.rs
├── contract/
│   ├── test_rust_api_contract.rs
│   └── test_python_api_contract.rs
└── common/
    └── mod.rs           # Test utilities and mocks

benches/
└── performance/
    ├── bench_latency.rs
    └── bench_throughput.rs

examples/
├── rust_example.rs      # Rust usage example
└── python_example.py    # Python usage example
```

**Structure Decision**: Single library project structure with clear module separation. The wrapper module contains core functionality, with separate modules for conversion, retry logic, authentication, and debug output. Python bindings are in a dedicated module. This structure enables code reuse from cap-gl-consumer-rust while maintaining clean separation of concerns.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations detected. All complexity is justified by requirements:
- Multi-language support (Rust + Python) is a core requirement
- OpenTelemetry integration is required for observability
- Debug file output is optional and disabled by default
- Retry logic and token refresh are required for production reliability

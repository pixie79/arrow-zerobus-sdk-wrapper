# Implementation Plan: Add Per-Row Error Information to TransmissionResult

**Branch**: `004-per-row-errors` | **Date**: 2025-01-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-per-row-errors/spec.md`

## Summary

Add per-row error tracking to `TransmissionResult` to enable partial batch success handling. This allows consumers to identify which specific rows failed during batch transmission, enabling efficient quarantine workflows that only quarantine failed rows while successfully writing valid rows to the main table. The implementation will extend the existing `TransmissionResult` struct with optional per-row error fields while maintaining backward compatibility.

## Technical Context

**Language/Version**: Rust edition 2021 (latest stable)  
**Primary Dependencies**: 
- `arrow` / `arrow-array` (v57) - Arrow data structures
- `databricks-zerobus-ingest-sdk` (v0.1.1) - Zerobus SDK
- `prost` / `prost-types` (v0.13) - Protobuf serialization
- `pyo3` (v0.20) - Python bindings
- `tokio` (v1.35) - Async runtime

**Storage**: N/A (in-memory data structures)  
**Testing**: `cargo test` (Rust native testing), `pytest` for Python bindings  
**Target Platform**: Cross-platform (Linux, macOS, Windows)  
**Project Type**: Single Rust library with Python bindings  
**Performance Goals**: 
- Per-row error tracking overhead < 10% compared to batch-level error handling
- Support batches of 100-20,000 rows efficiently
- Memory overhead for error tracking should be bounded

**Constraints**: 
- Must maintain backward compatibility (existing code continues to work)
- Must support both Rust and Python bindings
- Must work with existing retry logic
- Must handle edge cases (empty batches, all-success, all-failure scenarios)
- Performance impact must be minimal for successful batches

**Scale/Scope**: 
- Typical batch sizes: 100-20,000 rows
- Error tracking for up to 100% of rows in a batch
- Support for multiple error types per batch

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Research Gates

✅ **Code Quality Standards**: Feature extends existing struct with optional fields, maintaining clarity and idiomatic Rust patterns  
✅ **Testing Standards**: Will require comprehensive tests for per-row error scenarios (unit, integration, contract tests)  
✅ **User Experience Consistency**: Must maintain API consistency between Rust and Python bindings  
✅ **Performance Requirements**: Performance impact must be measured and validated (< 10% overhead target)  
✅ **Multi-Language Support**: Python bindings must support new per-row error fields  
✅ **Commit Workflow Standards**: All commits must follow workflow (CHANGELOG, formatting, tests, GPG signing)

**Status**: ✅ All gates pass - proceed to Phase 0 research

### Post-Design Gates (After Phase 1)

✅ **Code Quality Standards**: Design maintains idiomatic Rust patterns with optional fields, clear error handling, and comprehensive documentation  
✅ **Testing Standards**: Design includes test strategy for unit, integration, and contract tests covering all per-row error scenarios  
✅ **User Experience Consistency**: API contracts ensure consistent behavior between Rust and Python bindings with matching field semantics  
✅ **Performance Requirements**: Design includes performance targets (< 10% overhead) and efficient data structures (Vec with pre-allocation)  
✅ **Multi-Language Support**: Python bindings design includes native Python types (List, Tuple) with proper type hints  
✅ **Commit Workflow Standards**: All implementation commits must follow workflow (CHANGELOG, formatting, tests, GPG signing)

**Status**: ✅ All gates pass - ready for Phase 2 (task breakdown)

## Project Structure

### Documentation (this feature)

```text
specs/004-per-row-errors/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── rust-api.md      # Rust API contract
│   └── python-api.md    # Python API contract
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── wrapper/
│   ├── mod.rs           # TransmissionResult struct definition, send_batch methods
│   ├── conversion.rs    # Row-level conversion logic (needs modification)
│   └── zerobus.rs       # Transmission logic (needs modification)
├── error.rs             # ZerobusError enum (no changes needed)
└── python/
    └── bindings.rs      # Python bindings for TransmissionResult (needs modification)

tests/
├── unit/
│   └── wrapper/
│       └── test_per_row_errors.rs  # New unit tests
├── integration/
│   └── test_per_row_errors.rs      # New integration tests
└── contract/
    └── test_per_row_errors_contract.rs  # New contract tests
```

**Structure Decision**: Single Rust library project with Python bindings. Changes will be made to existing modules (`src/wrapper/mod.rs`, `src/wrapper/conversion.rs`, `src/wrapper/zerobus.rs`, `src/python/bindings.rs`) with new test files added to verify per-row error functionality.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations - feature extends existing structures with optional fields, maintaining simplicity and backward compatibility.

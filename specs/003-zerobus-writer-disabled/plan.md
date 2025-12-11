# Implementation Plan: Zerobus Writer Disabled Mode

**Branch**: `003-zerobus-writer-disabled` | **Date**: 2025-12-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-zerobus-writer-disabled/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add a configuration option to disable Zerobus SDK transmission while maintaining debug file output capabilities. This feature enables developers to test data conversion logic locally without network access, validate transformations in CI/CD pipelines without credentials, and benchmark conversion performance without network overhead. The implementation adds a boolean configuration flag that skips SDK initialization, stream creation, and data transmission calls while preserving Arrow-to-Protobuf conversion and debug file writing functionality.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021), Python 3.11+ for bindings

**Primary Dependencies**:
- Existing wrapper infrastructure (no new dependencies required)
- `databricks-zerobus-ingest-sdk` - Will be conditionally initialized based on configuration
- `arrow` / `arrow-array` (v57) - Arrow RecordBatch handling (already used)
- `prost` / `prost-types` (v0.13) - Protobuf serialization (already used)
- `pyo3` (v0.20) - Python bindings (already used)
- Debug file writing infrastructure (already implemented)

**Storage**: Local file system for debug output (already implemented, no changes needed)

**Testing**: 
- Rust: `cargo test` with native testing framework
- Python: `pytest` for Python bindings (Python 3.11+)
- Coverage: `cargo-tarpaulin` for Rust (≥90% per file requirement)
- Integration tests to verify no SDK calls when disabled
- Unit tests for configuration validation

**Target Platform**: Cross-platform (Linux, macOS, Windows) - no changes to platform support

**Project Type**: Single library project (Rust SDK with Python bindings) - no structural changes

**Performance Goals**:
- Operations complete in under 50ms when writer disabled mode is enabled (excluding file I/O time)
- Zero network overhead when disabled
- No performance impact when disabled mode is not used (default behavior unchanged)

**Constraints**:
- Must maintain ≥90% test coverage per file (constitution requirement)
- Must not break existing functionality (backward compatible)
- Configuration validation must prevent invalid combinations
- Debug output must be enabled when writer is disabled (or auto-enabled)
- Credentials should be optional when writer is disabled

**Scale/Scope**:
- Single configuration flag addition
- Modifications to existing wrapper initialization and batch sending logic
- No new modules required, only modifications to existing code
- Both Rust and Python interfaces must support the new option

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Code Quality Standards ✓
- ✅ Follows existing Rust patterns and idiomatic code
- ✅ No new unsafe code introduced
- ✅ Proper error handling for configuration validation
- ✅ Comprehensive documentation (rustdoc) for new configuration option

### Testing Standards ✓
- ✅ ≥90% test coverage per file (mandatory)
- ✅ TDD workflow (tests first, then implementation)
- ✅ Unit tests for configuration validation
- ✅ Integration tests to verify no SDK calls when disabled
- ✅ Tests for both Rust and Python interfaces

### User Experience Consistency ✓
- ✅ Consistent API across Rust and Python bindings
- ✅ Synchronized documentation
- ✅ Clear error messages for invalid configuration combinations
- ✅ Feature parity between interfaces

### Performance Requirements ✓
- ✅ No performance impact when disabled mode is not used
- ✅ Fast operation completion when disabled (<50ms excluding I/O)
- ✅ Zero network overhead when disabled

### Multi-Language Support Architecture ✓
- ✅ Rust core implementation
- ✅ PyO3 bindings for Python
- ✅ Shared underlying implementation
- ✅ Consistent configuration interface

**Status**: All constitution gates pass. No violations detected.

### Post-Design Constitution Check

After Phase 1 design completion:

**Code Quality Standards** ✓
- ✅ Minimal code changes following existing patterns
- ✅ Configuration validation follows existing error handling patterns
- ✅ Documentation requirements specified in API contracts

**Testing Standards** ✓
- ✅ Test structure defined (unit, integration)
- ✅ Coverage requirement (≥90%) maintained
- ✅ TDD workflow will be enforced during implementation

**User Experience Consistency** ✓
- ✅ API contracts defined for both Rust and Python with feature parity
- ✅ Consistent error types across both interfaces
- ✅ Quickstart guide provides examples for both languages

**Performance Requirements** ✓
- ✅ Performance targets specified (<50ms excluding I/O)
- ✅ No performance regression for normal operation
- ✅ Zero network overhead when disabled

**Multi-Language Support Architecture** ✓
- ✅ Rust core implementation with PyO3 bindings
- ✅ Shared implementation ensures consistency
- ✅ Configuration option available in both interfaces

**Status**: All constitution gates continue to pass after design phase. No violations detected.

## Project Structure

### Documentation (this feature)

```text
specs/003-zerobus-writer-disabled/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── rust-api.md      # Rust SDK API contract updates
│   └── python-api.md    # Python bindings API contract updates
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── config/
│   └── types.rs         # Add zerobus_writer_disabled field to WrapperConfiguration
├── wrapper/
│   └── mod.rs           # Modify send_batch_internal() to skip SDK calls when disabled
└── python/
    └── bindings.rs      # Add zerobus_writer_disabled parameter to PyWrapperConfiguration

tests/
├── unit/
│   ├── config/
│   │   └── test_types.rs    # Test configuration validation for disabled mode
│   └── wrapper/
│       └── test_zerobus.rs   # Test that SDK calls are skipped when disabled
└── integration/
    └── test_debug_files.rs  # Test debug file writing when writer is disabled
```

**Structure Decision**: This feature requires minimal structural changes. We're adding a single configuration field and modifying existing methods to conditionally skip SDK calls. No new modules or major refactoring required. The changes are localized to configuration types and the batch sending logic.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations detected. This feature adds minimal complexity:
- Single boolean configuration flag
- Conditional logic in existing methods (early return pattern)
- No new dependencies or architectural changes
- Backward compatible (default behavior unchanged)

## Phase 0: Research & Decisions

See [research.md](./research.md) for detailed research findings and decisions.

### Key Decisions

1. **Configuration Validation**: When `zerobus_writer_disabled` is true, `debug_enabled` must also be true. Auto-enabling debug when writer is disabled provides better UX than requiring explicit configuration.

2. **Credentials Handling**: Credentials become optional when writer is disabled, allowing local development without Databricks access.

3. **Return Value**: `TransmissionResult` will indicate success when writer is disabled, with optional metadata indicating debug-only mode.

4. **Early Return Pattern**: Use early return in `send_batch_internal()` after debug file writing to skip all SDK-related code paths.

## Phase 1: Design & Contracts

See [data-model.md](./data-model.md), [contracts/](./contracts/), and [quickstart.md](./quickstart.md) for detailed design artifacts.

### Data Model Changes

- **WrapperConfiguration**: Add `zerobus_writer_disabled: bool` field (default: false)
- **TransmissionResult**: No changes required (existing success/error structure sufficient)

### API Contract Changes

- **Rust API**: Add `with_zerobus_writer_disabled(bool)` builder method
- **Python API**: Add `zerobus_writer_disabled: bool = False` parameter

### Implementation Approach

1. Add configuration field with validation
2. Modify `send_batch_internal()` to check flag and return early after debug writing
3. Skip SDK initialization, stream creation, and transmission when disabled
4. Update Python bindings to expose the new option
5. Add comprehensive tests

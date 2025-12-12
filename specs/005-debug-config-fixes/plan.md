# Implementation Plan: Debug Output Configuration Fixes

**Branch**: `005-debug-config-fixes` | **Date**: 2025-12-12 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-debug-config-fixes/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

This feature addresses three areas related to debug output configuration:

1. **Separate Arrow/Protobuf Debug Flags (Issue #14)**: Add independent configuration flags (`debug_arrow_enabled` and `debug_protobuf_enabled`) to allow users to enable Arrow and Protobuf debug output independently, replacing the single `debug_enabled` flag while maintaining backward compatibility.

2. **Fix Recursive Timestamp Appending (Issue #13)**: Fix the file rotation logic to prevent recursive timestamp appending in rotated file names by extracting the original base file name (without timestamps) before generating new rotated paths.

3. **Automatic File Retention Management**: Implement configurable file retention limits (default: 10 files per type) with automatic cleanup of oldest files when limit is exceeded, preventing unlimited disk space consumption.

**Technical Approach**: 
- Extend `WrapperConfiguration` with new boolean fields for Arrow and Protobuf debug flags, plus retention limit configuration
- Update configuration loaders (YAML, environment variables, programmatic API) to support new flags and retention settings
- Modify `DebugWriter` to conditionally write Arrow/Protobuf files based on respective flags
- Fix `generate_rotated_path()` to extract base filename without timestamps before appending new timestamp
- Implement file retention cleanup logic that deletes oldest rotated files immediately after rotation when limit exceeded
- Maintain backward compatibility: if old `debug_enabled` is set, enable both formats by default

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2024)  
**Primary Dependencies**: 
- `arrow` (v57) - Arrow IPC Stream format for debug files
- `prost` (v0.13) - Protobuf serialization
- `chrono` (v0.4) - Timestamp formatting for file rotation
- `tokio` (v1.35) - Async runtime for file I/O
- `serde` (v1.0) - Configuration serialization (YAML)
- `pyo3` (v0.20) - Python bindings support
- `regex` (v1.x) - Pattern matching for timestamp extraction from filenames

**Storage**: File system (debug output files in user-specified directory)  
**Testing**: `cargo test` (unit, integration, contract tests)  
**Target Platform**: Cross-platform (Linux, macOS, Windows)  
**Project Type**: Single Rust library with Python bindings  
**Performance Goals**: 
- Configuration changes take effect immediately (no restart required)
- File rotation overhead <1ms per rotation
- File retention cleanup overhead <5ms per cleanup operation
- No performance impact when debug output is disabled

**Constraints**: 
- Must maintain backward compatibility with existing `debug_enabled` configuration
- File names must not exceed 255 characters (POSIX filename limit)
- Configuration must support YAML, environment variables, and programmatic API
- Both Rust and Python bindings must support new flags and retention settings
- File deletion failures should not block file rotation (log and continue)

**Scale/Scope**: 
- Affects configuration system, debug writer, and file rotation utilities
- Changes span ~6 files: `src/config/types.rs`, `src/config/loader.rs`, `src/wrapper/debug.rs`, `src/wrapper/mod.rs`, `src/python/bindings.rs`, `src/utils/file_rotation.rs`
- Requires updates to tests in `tests/unit/config/`, `tests/unit/wrapper/`, `tests/integration/`
- New file retention logic requires directory scanning and file deletion capabilities

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Code Quality Standards (NON-NEGOTIABLE)
- ✅ **Status**: PASS
- **Rationale**: Changes follow existing patterns, maintain idiomatic Rust, and require comprehensive documentation updates. No unsafe code introduced. File deletion uses safe filesystem APIs.

### Testing Standards (NON-NEGOTIABLE)
- ✅ **Status**: PASS
- **Rationale**: Feature requires unit tests for configuration loading, integration tests for debug file writing and retention cleanup, and contract tests for API changes. TDD workflow will be followed. Coverage must remain ≥90%.

### User Experience Consistency
- ✅ **Status**: PASS
- **Rationale**: Both Rust and Python bindings will support new flags and retention settings with identical semantics. Backward compatibility ensures existing users are unaffected.

### Performance Requirements
- ✅ **Status**: PASS
- **Rationale**: File retention cleanup happens asynchronously after rotation, minimizing impact. File deletion operations are O(n) where n is number of files, but n is bounded by retention limit (default 10). Performance impact is minimal and bounded.

### Multi-Language Support Architecture
- ✅ **Status**: PASS
- **Rationale**: Changes maintain Rust core implementation with Python bindings via PyO3. Both interfaces will support new flags and retention settings identically.

### Commit Workflow Standards (NON-NEGOTIABLE)
- ✅ **Status**: PASS
- **Rationale**: All commits will follow workflow: CHANGELOG.md updated, documentation updated, `cargo fmt`, `cargo clippy` passing, all tests passing, GPG signed.

**Overall Gate Status**: ✅ **PASS** - All constitution requirements met. Proceeding to Phase 0 research.

---

## Post-Phase 1 Design Re-Evaluation

### Code Quality Standards (NON-NEGOTIABLE)
- ✅ **Status**: PASS
- **Rationale**: Design maintains idiomatic Rust patterns, extends existing structures without breaking changes, and includes comprehensive documentation updates. File retention logic uses safe filesystem operations.

### Testing Standards (NON-NEGOTIABLE)
- ✅ **Status**: PASS
- **Rationale**: Design includes test strategy for configuration loading, file rotation, file retention cleanup, and API contracts. TDD workflow will be followed. Coverage requirements maintained.

### User Experience Consistency
- ✅ **Status**: PASS
- **Rationale**: Both Rust and Python APIs maintain semantic equivalence. Backward compatibility ensures existing users unaffected. API contracts document consistent behavior.

### Performance Requirements
- ✅ **Status**: PASS
- **Rationale**: File retention cleanup is asynchronous and bounded by retention limit. No performance-critical paths affected. Configuration checks are O(1).

### Multi-Language Support Architecture
- ✅ **Status**: PASS
- **Rationale**: Changes maintain Rust core with Python bindings. Both interfaces will support new flags and retention settings identically.

### Commit Workflow Standards (NON-NEGOTIABLE)
- ✅ **Status**: PASS
- **Rationale**: All commits will follow workflow standards. CHANGELOG.md updates, documentation updates, formatting, linting, testing, and GPG signing required.

**Post-Design Gate Status**: ✅ **PASS** - All constitution requirements continue to be met after Phase 1 design.

## Project Structure

### Documentation (this feature)

```text
specs/005-debug-config-fixes/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── config/
│   ├── loader.rs          # Configuration loading (YAML, env vars) - MODIFIED
│   ├── types.rs           # Configuration structs - MODIFIED
│   └── mod.rs
├── wrapper/
│   ├── debug.rs           # Debug file writer - MODIFIED (rotation fix, retention)
│   ├── mod.rs             # Wrapper initialization - MODIFIED (separate flags)
│   ├── zerobus.rs
│   └── ...
├── python/
│   └── bindings.rs        # Python bindings - MODIFIED (new flags, retention)
└── lib.rs

tests/
├── unit/
│   ├── config/
│   │   └── test_loader.rs  # MODIFIED (test new flags, retention)
│   └── wrapper/
│       └── test_debug.rs    # MODIFIED (test rotation fix, retention)
├── integration/
│   └── test_debug_files.rs # MODIFIED (test separate flags, retention)
└── contract/
    └── test_rust_api_contract.rs  # MODIFIED (verify API changes)
```

**Structure Decision**: Single Rust library project with Python bindings. Changes are localized to configuration system and debug writer module. File retention logic is integrated into `DebugWriter` struct. No new modules required.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**No violations**: All constitution requirements are met. No complexity justification needed.

---

## Phase Completion Summary

### Phase 0: Research ✅ COMPLETE

**Deliverables**:
- ✅ `research.md` - Technical decisions documented
  - Separate flags implementation approach
  - File rotation timestamp extraction strategy
  - Configuration loading precedence rules
  - Python bindings API design
  - File retention implementation approach
  - Testing strategy

**Key Decisions**:
- Add two boolean fields (`debug_arrow_enabled`, `debug_protobuf_enabled`) to configuration
- Add configurable retention limit (`debug_max_files_retained`) with default 10, allow 0 for unlimited
- Use regex-based timestamp extraction to prevent recursion
- Implement file retention cleanup immediately after rotation
- Maintain backward compatibility with legacy `debug_enabled` flag
- Optional Python parameters with `None` defaults

### Phase 1: Design & Contracts ✅ COMPLETE

**Deliverables**:
- ✅ `data-model.md` - Entity definitions and data flow
  - DebugConfiguration entity with new fields and retention settings
  - FileRotationState tracking
  - BaseFileName extraction logic
  - File retention cleanup flow
- ✅ `contracts/rust-api.md` - Rust API contract
  - Configuration struct changes
  - Builder methods for new flags and retention
  - Configuration loading precedence
  - File rotation API changes
  - File retention API
- ✅ `contracts/python-api.md` - Python API contract
  - Updated `__init__` signature with new optional parameters
  - Configuration precedence rules
  - File retention configuration
  - Backward compatibility examples
- ✅ `quickstart.md` - Usage guide
  - Quick start examples (Rust & Python)
  - Configuration methods
  - File retention examples
  - Common use cases
  - Migration guide
- ✅ Agent context updated - Cursor IDE context file updated with feature details

**Design Artifacts**:
- Configuration API design complete
- File rotation fix approach documented
- File retention implementation approach documented
- Backward compatibility strategy defined
- Test strategy outlined

### Next Phase: Implementation Planning

**Ready for**: `/speckit.tasks` command to break down implementation into specific tasks.

**Estimated Scope**:
- ~6 source files to modify
- ~4 test files to update/add
- Configuration, debug writer, file retention, and Python bindings changes
- Comprehensive test coverage required (≥90%)

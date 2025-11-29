# Implementation Plan: OTLP SDK Integration Update

**Branch**: `002-otlp-sdk-update` | **Date**: 2025-01-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-otlp-sdk-update/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Update the observability implementation to use the SDK from otlp-rust-service for metrics collection and logging via traces, replacing manual construction of observability data structures. Remove all dead code including manual construction methods (create_batch_metrics, create_span_data, convert_config) and the synchronous `new()` method. Allow breaking changes to both API and configuration since no users currently depend on the wrapper.

## Technical Context

**Language/Version**: Rust 2021 edition (current stable)  
**Primary Dependencies**: 
- otlp-arrow-library (from otlp-rust-service, main branch) - for SDK-based observability
- opentelemetry = "0.31" - OpenTelemetry API compatibility
- opentelemetry_sdk = "0.31" - OpenTelemetry SDK
- tokio = "1.35" - Async runtime
- tracing = "0.1" - Structured logging

**Storage**: N/A (observability data exported via SDK)  
**Testing**: cargo test with tokio-test for async testing  
**Target Platform**: Cross-platform (Linux, macOS, Windows)  
**Project Type**: Single Rust library with optional Python bindings  
**Performance Goals**: 
- SDK initialization succeeds in 99.9% of cases
- Metrics and traces exported with same reliability as previous implementation
- No performance degradation from SDK usage

**Constraints**: 
- Must maintain async initialization pattern
- Must handle SDK initialization failures gracefully
- Must not break data transmission operations if observability fails
- Breaking changes allowed (no current users)

**Scale/Scope**: 
- Single observability module (src/observability/otlp.rs)
- Remove ~150 lines of dead code (manual construction methods)
- Update configuration structure to align with SDK requirements

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: PASSED

**Rationale**: 
- Single feature update to existing module
- No new dependencies beyond SDK update
- Removing code (dead code elimination) reduces complexity
- Breaking changes acceptable (no current users)
- No architectural changes required

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
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
│   ├── loader.rs
│   ├── mod.rs
│   └── types.rs          # OtlpConfig structure (to be updated)
├── observability/
│   ├── mod.rs
│   └── otlp.rs           # ObservabilityManager (main update target)
├── wrapper/
│   └── [other modules]
└── lib.rs

tests/
├── integration/
│   └── test_observability.rs  # Tests to be updated
└── unit/
    └── observability/
        └── test_otlp.rs       # Unit tests to be updated
```

**Structure Decision**: Single Rust library project. The observability module (`src/observability/otlp.rs`) is the primary target for updates. Configuration types in `src/config/types.rs` will be updated to align with SDK requirements. Tests in `tests/integration/test_observability.rs` and `tests/unit/observability/test_otlp.rs` will be updated to reflect the new SDK-based API.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations - this is a simplification effort (removing dead code and using SDK instead of manual construction).

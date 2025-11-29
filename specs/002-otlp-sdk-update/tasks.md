# Tasks: OTLP SDK Integration Update

**Input**: Design documents from `/specs/002-otlp-sdk-update/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are recommended to verify SDK integration. Test tasks are included to ensure SDK-based implementation works correctly.

**Organization**: Tasks are organized by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below follow the single project structure from plan.md

## Phase 1: Setup (Project Initialization)

**Purpose**: Project initialization and dependency updates

- [X] T001 Update Cargo.toml to ensure otlp-arrow-library dependency is configured correctly from otlp-rust-service main branch
- [X] T002 [P] Verify OpenTelemetry dependencies (opentelemetry = "0.31", opentelemetry_sdk = "0.31") are compatible with SDK requirements
- [X] T003 [P] Review current observability module structure in src/observability/otlp.rs to understand existing implementation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Create OtlpSdkConfig struct in src/config/types.rs with fields: endpoint, output_dir, write_interval_secs, log_level per data-model.md
- [X] T005 [P] Update WrapperConfiguration struct in src/config/types.rs to use OtlpSdkConfig instead of OtlpConfig for observability_config field
- [X] T006 [P] Add validation for OtlpSdkConfig in src/config/types.rs (endpoint URL validation, output_dir path validation, write_interval_secs > 0, valid log_level)
- [X] T007 [P] Update configuration loader in src/config/loader.rs to support OtlpSdkConfig deserialization (if loader handles observability config)
- [X] T008 [P] Review otlp-rust-service SDK API documentation or source code to identify exact methods for metrics and trace creation

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Use OTLP Service SDK for Metrics and Traces (Priority: P1) 🎯 MVP

**Goal**: Update the wrapper to use the standardized SDK from otlp-rust-service for metrics collection and logging via traces, replacing manual construction of observability data structures.

**Independent Test**: Can be fully tested by initializing the wrapper with observability enabled, performing data transmission operations, and verifying that metrics and traces are generated using the otlp-rust-service SDK rather than manual construction. The test delivers value by confirming the wrapper uses the standardized SDK approach.

### Tests for User Story 1

> **NOTE: Write these tests to verify SDK integration works correctly.**

- [X] T009 [P] [US1] Unit tests for OtlpSdkConfig validation in tests/unit/config/test_types.rs (verify endpoint, output_dir, write_interval_secs, log_level validation)
- [X] T010 [P] [US1] Unit tests for ObservabilityManager initialization with OtlpSdkConfig in tests/unit/observability/test_otlp.rs (verify SDK initialization succeeds/fails correctly)
- [X] T011 [P] [US1] Unit tests for SDK-based metrics recording in tests/unit/observability/test_otlp.rs (verify record_batch_sent uses SDK methods)
- [X] T012 [P] [US1] Unit tests for SDK-based trace creation in tests/unit/observability/test_otlp.rs (verify start_send_batch_span uses SDK methods)
- [X] T013 [US1] Integration test for SDK-based observability in tests/integration/test_observability.rs (verify metrics and traces exported via SDK)

### Implementation for User Story 1

- [X] T014 [US1] Remove synchronous `new()` method from ObservabilityManager in src/observability/otlp.rs (dead code that always returns None)
- [X] T015 [US1] Update `new_async()` method signature in src/observability/otlp.rs to accept `Option<OtlpSdkConfig>` instead of `Option<OtlpConfig>`
- [X] T016 [US1] Remove `convert_config()` method from ObservabilityManager in src/observability/otlp.rs (replaced by direct SDK config usage)
- [X] T017 [US1] Update `new_async()` implementation in src/observability/otlp.rs to use SDK ConfigBuilder directly with OtlpSdkConfig fields (remove conversion layer)
- [X] T018 [US1] Remove `create_batch_metrics()` method from ObservabilityManager in src/observability/otlp.rs (replaced by SDK methods)
- [X] T019 [US1] Update `record_batch_sent()` method in src/observability/otlp.rs to use SDK methods for metrics recording instead of manual ResourceMetrics construction
- [X] T020 [US1] Remove `create_span_data()` method from ObservabilitySpan in src/observability/otlp.rs (replaced by SDK methods)
- [X] T021 [US1] Update `start_send_batch_span()` method in src/observability/otlp.rs to use SDK for span creation instead of manual SpanData construction
- [X] T022 [US1] Update ObservabilitySpan Drop implementation in src/observability/otlp.rs to use SDK methods for span ending and export instead of manual SpanData construction
- [X] T023 [US1] Update ObservabilitySpan struct in src/observability/otlp.rs to use SDK span type instead of manual span data (if SDK provides span guard types)
- [X] T024 [US1] Update all imports in src/observability/otlp.rs to use OtlpSdkConfig instead of OtlpConfig
- [X] T025 [US1] Update WrapperConfiguration usage in src/wrapper/mod.rs to use OtlpSdkConfig (if wrapper initializes observability)
- [X] T026 [US1] Verify graceful error handling in src/observability/otlp.rs (SDK initialization failures log warnings and continue without observability)
- [X] T027 [US1] Verify export error handling in src/observability/otlp.rs (export failures log errors but don't interrupt operations)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. The wrapper uses the otlp-rust-service SDK for all metrics and trace operations.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Improvements and validation that affect the implementation

- [X] T028 [P] Update rustdoc documentation in src/observability/otlp.rs to reflect SDK-based API (remove references to manual construction)
- [X] T029 [P] Update rustdoc documentation in src/config/types.rs to document OtlpSdkConfig structure and migration from OtlpConfig
- [X] T030 [P] Verify all dead code is removed (confirm create_batch_metrics, create_span_data, convert_config, and synchronous new() are deleted)
- [X] T031 [P] Run cargo clippy and fix any warnings introduced by SDK integration changes
- [X] T032 [P] Run cargo fmt to ensure code formatting is consistent
- [X] T033 [P] Verify SDK initialization succeeds in 99.9% of cases when valid configuration is provided (test with various config combinations)
- [X] T034 [P] Verify metrics and traces are exported successfully using SDK methods with same reliability as previous implementation (integration test validation)
- [X] T035 [P] Update quickstart.md examples if needed to reflect OtlpSdkConfig usage (if quickstart.md has observability examples)
- [X] T036 [P] Verify breaking changes are documented (OtlpConfig → OtlpSdkConfig migration path if applicable)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS user story implementation
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion
- **Polish (Phase 4)**: Depends on User Story 1 completion

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories

### Within User Story 1

- Configuration updates (T004-T007) before observability manager updates
- SDK API review (T008) should inform implementation approach
- Tests can be written in parallel after foundational phase
- Dead code removal can happen in parallel with SDK integration
- Core SDK integration (T014-T027) should follow logical order:
  1. Remove dead code
  2. Update configuration types
  3. Update initialization
  4. Update metrics recording
  5. Update trace creation

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes:
  - All test tasks for US1 marked [P] can run in parallel
  - Dead code removal tasks can run in parallel with SDK integration tasks (different methods)
  - Configuration updates and observability updates can proceed in logical order
- All Polish tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch foundational tasks in parallel:
Task: "Create OtlpSdkConfig struct in src/config/types.rs"
Task: "Update WrapperConfiguration struct in src/config/types.rs"
Task: "Add validation for OtlpSdkConfig in src/config/types.rs"
Task: "Review otlp-rust-service SDK API documentation"

# Launch test tasks in parallel (after foundational):
Task: "Unit tests for OtlpSdkConfig validation in tests/unit/config/test_types.rs"
Task: "Unit tests for ObservabilityManager initialization in tests/unit/observability/test_otlp.rs"
Task: "Unit tests for SDK-based metrics recording in tests/unit/observability/test_otlp.rs"
Task: "Unit tests for SDK-based trace creation in tests/unit/observability/test_otlp.rs"

# Launch implementation tasks (after tests, in logical order):
Task: "Remove synchronous new() method from ObservabilityManager in src/observability/otlp.rs"
Task: "Remove convert_config() method from ObservabilityManager in src/observability/otlp.rs"
Task: "Remove create_batch_metrics() method from ObservabilityManager in src/observability/otlp.rs"
Task: "Remove create_span_data() method from ObservabilitySpan in src/observability/otlp.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify dependencies)
2. Complete Phase 2: Foundational (CRITICAL - blocks user story)
3. Complete Phase 3: User Story 1 (SDK integration)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (SDK integration complete)
3. Each phase adds value without breaking previous functionality

### Implementation Approach

1. **Dead Code Removal First**: Remove unused methods to simplify codebase
2. **Configuration Update**: Update config types to align with SDK
3. **SDK Integration**: Replace manual construction with SDK methods
4. **Testing**: Verify SDK-based implementation works correctly
5. **Polish**: Documentation and validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- User Story 1 should be independently completable and testable
- **RECOMMENDED**: Write tests to verify SDK integration works correctly
- Commit after each task or logical group
- Stop at checkpoint to validate story independently
- Breaking changes are allowed (no current users per spec)
- SDK API methods will be determined during implementation by examining otlp-rust-service SDK source code or documentation
- All dead code must be removed: new(), create_batch_metrics(), create_span_data(), convert_config()
- Configuration structure simplified: OtlpConfig → OtlpSdkConfig (breaking change)
- Maintain graceful error handling: SDK failures should not interrupt data transmission operations


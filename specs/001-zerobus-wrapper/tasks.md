# Tasks: Zerobus SDK Wrapper

**Input**: Design documents from `/specs/001-zerobus-wrapper/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are MANDATORY per constitution - ≥90% coverage per file required. All test tasks must be completed before implementation tasks.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below follow the single project structure from plan.md

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create project structure per implementation plan (src/, tests/, benches/, examples/)
- [X] T002 Initialize Rust project with Cargo.toml, edition 2021, and all dependencies from plan.md
- [X] T003 [P] Configure rustfmt.toml and clippy.toml with project standards
- [X] T004 [P] Setup test coverage tooling (cargo-tarpaulin) with 90% threshold enforcement in CI
- [X] T005 [P] Create README.md with project overview and build instructions
- [X] T006 [P] Setup CI/CD pipeline configuration for Rust and Python testing

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T007 Create error types in src/error.rs with ZerobusError enum and all variants from data-model.md
- [X] T008 [P] Create configuration types in src/config/types.rs with WrapperConfiguration struct and all fields
- [X] T009 [P] Implement configuration validation in src/config/types.rs (validate method per data-model.md rules)
- [X] T010 [P] Implement configuration loader in src/config/loader.rs (YAML and environment variable support)
- [X] T011 Create RetryConfig struct in src/wrapper/retry.rs with exponential backoff + jitter logic
- [X] T012 [P] Create test utilities and mocks in tests/common/mod.rs for shared test infrastructure
- [X] T013 [P] Setup unit test structure in tests/unit/ with subdirectories for each module
- [X] T014 [P] Setup integration test structure in tests/integration/ with test files per user story
- [X] T015 [P] Setup contract test structure in tests/contract/ for API contract validation

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Send Data to Zerobus from Rust Application (Priority: P1) 🎯 MVP

**Goal**: Enable Rust applications to send Arrow RecordBatch data to Zerobus with automatic protocol conversion, authentication, and retry logic

**Independent Test**: Create a Rust application that initializes the wrapper, sends a batch of test data, and verifies successful delivery to Zerobus. The test delivers value by confirming the wrapper can successfully transmit data.

### Tests for User Story 1 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [X] T016 [P] [US1] Unit tests for error types in tests/unit/error/test_error.rs (target ≥90% coverage)
- [X] T017 [P] [US1] Unit tests for configuration types in tests/unit/config/test_types.rs (target ≥90% coverage)
- [X] T018 [P] [US1] Unit tests for configuration loader in tests/unit/config/test_loader.rs (target ≥90% coverage)
- [X] T019 [P] [US1] Unit tests for retry logic in tests/unit/wrapper/test_retry.rs (target ≥90% coverage)
- [X] T020 [P] [US1] Unit tests for Arrow to Protobuf conversion in tests/unit/wrapper/test_conversion.rs (target ≥90% coverage)
- [X] T021 [P] [US1] Unit tests for Zerobus SDK integration in tests/unit/wrapper/test_zerobus.rs (target ≥90% coverage)
- [X] T022 [P] [US1] Unit tests for authentication and token refresh in tests/unit/wrapper/test_auth.rs (target ≥90% coverage)
- [X] T023 [P] [US1] Contract test for Rust API in tests/contract/test_rust_api_contract.rs (verify API contract compliance)
- [X] T024 [US1] Integration test for Rust API end-to-end in tests/integration/test_rust_api.rs (full user journey)

### Implementation for User Story 1

- [X] T025 [US1] Implement Arrow to Protobuf conversion in src/wrapper/conversion.rs (reuse patterns from cap-gl-consumer-rust)
- [X] T026 [US1] Implement retry logic with exponential backoff + jitter in src/wrapper/retry.rs (custom implementation per research.md)
- [X] T027 [US1] Implement authentication and token refresh in src/wrapper/auth.rs (transparent refresh on expiration)
- [X] T028 [US1] Implement Zerobus SDK integration in src/wrapper/zerobus.rs (reuse ZerobusSDKWriter patterns from cap-gl-consumer-rust)
- [X] T029 [US1] Implement main wrapper struct ZerobusWrapper in src/wrapper/mod.rs (thread-safe with Arc<tokio::sync::Mutex>)
- [X] T030 [US1] Implement send_batch method in src/wrapper/mod.rs (accepts RecordBatch, converts, transmits with retry)
- [X] T031 [US1] Implement flush and shutdown methods in src/wrapper/mod.rs (graceful cleanup)
- [X] T032 [US1] Create TransmissionResult struct in src/wrapper/mod.rs (per data-model.md)
- [X] T033 [US1] Export public API from src/lib.rs (ZerobusWrapper, WrapperConfiguration, ZerobusError, TransmissionResult)
- [ ] T034 [US1] Add comprehensive rustdoc documentation for all public APIs in src/lib.rs and src/wrapper/mod.rs
- [ ] T035 [US1] Verify test coverage ≥90% for all new files (cargo-tarpaulin)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Rust applications can send data to Zerobus.

---

## Phase 4: User Story 2 - Send Data to Zerobus from Python Application (Priority: P1)

**Goal**: Enable Python applications to send Arrow RecordBatch data to Zerobus by calling the Rust SDK through Python bindings with Pythonic API design

**Independent Test**: Create a Python application that imports the wrapper, initializes it with configuration, sends a batch of test data, and verifies successful delivery. The test delivers value by confirming Python developers can use the wrapper effectively.

### Tests for User Story 2 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [X] T036 [P] [US2] Unit tests for Python bindings in tests/unit/python/test_bindings.rs (target ≥90% coverage)
- [X] T037 [P] [US2] Python integration tests in tests/python/test_integration.py (verify Python API works)
- [ ] T038 [P] [US2] Contract test for Python API in tests/contract/test_python_api_contract.rs (verify API contract compliance)
- [X] T039 [US2] Integration test for Python bindings end-to-end in tests/integration/test_python_bindings.rs (full user journey)

### Implementation for User Story 2

- [X] T040 [US2] Implement PyO3 bindings for WrapperConfiguration in src/python/bindings.rs (Python 3.11+ support)
- [X] T041 [US2] Implement PyO3 bindings for ZerobusWrapper in src/python/bindings.rs (async methods with tokio runtime)
- [X] T042 [US2] Implement PyO3 bindings for TransmissionResult in src/python/bindings.rs (Python dataclass)
- [X] T043 [US2] Implement PyO3 bindings for ZerobusError in src/python/bindings.rs (Python exception classes)
- [X] T044 [US2] Implement zero-copy Arrow data transfer in src/python/bindings.rs (PyArrow integration)
- [X] T045 [US2] Export Python module in src/python/mod.rs (pyo3 module definition)
- [X] T046 [US2] Configure PyO3 build in Cargo.toml (extension-module, auto-initialize features)
- [X] T047 [US2] Create Python package structure (pyproject.toml, setup.py or maturin configuration)
- [X] T048 [US2] Add Python docstrings for all public APIs in src/python/bindings.rs (synchronized with Rust docs)
- [X] T049 [US2] Create Python usage example in examples/python_example.py (per quickstart.md)
- [X] T050 [US2] Verify test coverage ≥90% for all new files (cargo-tarpaulin for Rust, coverage.py for Python)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Both Rust and Python applications can send data to Zerobus.

---

## Phase 5: User Story 3 - Monitor Operations with OpenTelemetry Observability (Priority: P2)

**Goal**: Provide visibility into wrapper operations through standardized OpenTelemetry metrics and traces using otlp-rust-service library

**Independent Test**: Initialize the wrapper with observability enabled, perform data transmission operations, and verify that metrics and traces are generated and can be exported to standard observability backends. The test delivers value by confirming operational visibility is available.

### Tests for User Story 3 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [X] T051 [P] [US3] Unit tests for observability integration in tests/unit/observability/test_otlp.rs (target ≥90% coverage)
- [X] T052 [P] [US3] Integration test for observability in tests/integration/test_observability.rs (verify metrics and traces exported)

### Implementation for User Story 3

- [X] T053 [US3] Integrate otlp-rust-service library dependency in Cargo.toml (local path dependency)
- [X] T054 [US3] Implement observability module in src/observability/mod.rs (wrapper around OtlpLibrary)
- [X] T055 [US3] Implement OpenTelemetry integration in src/observability/otlp.rs (use OtlpLibrary API per research.md)
- [X] T056 [US3] Add metrics collection in src/wrapper/mod.rs (operation counts, latencies, success/failure rates)
- [X] T057 [US3] Add trace collection in src/wrapper/mod.rs (operation flow and timing information)
- [X] T058 [US3] Configure observability in WrapperConfiguration (observability_enabled, observability_config fields)
- [X] T059 [US3] Initialize observability in ZerobusWrapper::new (create OtlpLibrary instance if enabled)
- [X] T060 [US3] Export observability data on operation completion (per API contract)
- [X] T061 [US3] Add observability configuration to Python bindings in src/python/bindings.rs
- [ ] T062 [US3] Verify test coverage ≥90% for all new files (cargo-tarpaulin)

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently. Observability is available when enabled.

---

## Phase 6: User Story 4 - Debug Data Flow with File Output (Priority: P3)

**Goal**: Enable developers to debug data transformation issues by inspecting Arrow RecordBatch and Protobuf files written to local disk (optional, disabled by default)

**Independent Test**: Enable debug file output, perform data transmission operations, and verify that Arrow and Protobuf files are written to the configured output directory with the expected content. The test delivers value by confirming developers can inspect data transformations.

### Tests for User Story 4 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [ ] T063 [P] [US4] Unit tests for debug writer in tests/unit/wrapper/test_debug.rs (target ≥90% coverage)
- [ ] T064 [P] [US4] Unit tests for file rotation in tests/unit/utils/test_file_rotation.rs (target ≥90% coverage)
- [ ] T065 [US4] Integration test for debug file output in tests/integration/test_debug_files.rs (verify files written correctly)

### Implementation for User Story 4

- [ ] T066 [US4] Implement file rotation utility in src/utils/file_rotation.rs (size-based rotation per research.md)
- [ ] T067 [US4] Implement debug writer in src/wrapper/debug.rs (DebugWriter struct per data-model.md)
- [ ] T068 [US4] Implement Arrow IPC file writing in src/wrapper/debug.rs (write RecordBatch to {OUTPUT_DIR}/zerobus/arrow/table.arrow)
- [ ] T069 [US4] Implement Protobuf file writing in src/wrapper/debug.rs (write Protobuf to {OUTPUT_DIR}/zerobus/proto/table.proto)
- [ ] T070 [US4] Implement periodic flushing in src/wrapper/debug.rs (flush every 5 seconds configurable, default 5s)
- [ ] T071 [US4] Integrate debug writer in ZerobusWrapper (create DebugWriter if debug_enabled in config)
- [ ] T072 [US4] Write Arrow files during send_batch in src/wrapper/mod.rs (if debug enabled)
- [ ] T073 [US4] Write Protobuf files during conversion in src/wrapper/conversion.rs (if debug enabled)
- [ ] T074 [US4] Add debug configuration to WrapperConfiguration (debug_enabled, debug_output_dir, debug_flush_interval_secs, debug_max_file_size)
- [ ] T075 [US4] Add debug configuration to Python bindings in src/python/bindings.rs
- [ ] T076 [US4] Verify test coverage ≥90% for all new files (cargo-tarpaulin)
- [ ] T077 [US4] Verify zero performance impact when debug disabled (performance test)

**Checkpoint**: At this point, all user stories should be independently functional. Debug file output is available when enabled.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T078 [P] Documentation updates in README.md (rustdoc + Python docs, usage examples)
- [ ] T079 [P] Code cleanup and refactoring (maintain ≥90% coverage)
- [ ] T080 [P] Performance optimization across all stories (benchmark validation)
- [X] T081 [P] Create performance benchmarks in benches/performance/bench_latency.rs (p95 < 150ms target)
- [X] T082 [P] Create performance benchmarks in benches/performance/bench_throughput.rs (99.999% success rate)
- [ ] T083 [P] Verify ≥90% test coverage for all files (cargo-tarpaulin report)
- [X] T084 [P] Python binding documentation and examples (synchronize with Rust docs)
- [X] T085 [P] Create Rust usage example in examples/rust_example.rs (per quickstart.md)
- [ ] T086 [P] Run quickstart.md validation (both Rust and Python examples work)
- [ ] T087 [P] Cross-platform testing (Linux, macOS, Windows)
- [ ] T088 [P] Security review and hardening
- [ ] T089 [P] Final integration testing with cap-gl-consumer-rust compatibility (verify drop-in replacement)
- [X] T090 [P] Create README section for DuckDB usage to read Arrow and Protobuf debug files (examples for both formats)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Depends on US1 for shared Rust SDK implementation
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Depends on US1 for wrapper operations to instrument
- **User Story 4 (P3)**: Can start after Foundational (Phase 2) - Depends on US1 for data flow to debug

### Within Each User Story

- Tests (MANDATORY) MUST be written and FAIL before implementation
- Core modules before wrapper integration
- Wrapper implementation before Python bindings
- Core implementation before observability/debug integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes:
  - US1 core implementation can start
  - US2 can start after US1 core is complete (needs Rust SDK)
  - US3 can start after US1 is complete (needs wrapper to instrument)
  - US4 can start after US1 is complete (needs data flow to debug)
- All tests for a user story marked [P] can run in parallel
- Different modules within a story marked [P] can run in parallel (if no dependencies)

---

## Parallel Example: User Story 1

```bash
# Launch all unit tests for User Story 1 together:
Task: "Unit tests for error types in tests/unit/error/test_error.rs"
Task: "Unit tests for configuration types in tests/unit/config/test_types.rs"
Task: "Unit tests for configuration loader in tests/unit/config/test_loader.rs"
Task: "Unit tests for retry logic in tests/unit/wrapper/test_retry.rs"
Task: "Unit tests for Arrow to Protobuf conversion in tests/unit/wrapper/test_conversion.rs"

# Launch core implementation modules in parallel (after tests):
Task: "Implement Arrow to Protobuf conversion in src/wrapper/conversion.rs"
Task: "Implement retry logic with exponential backoff + jitter in src/wrapper/retry.rs"
Task: "Implement authentication and token refresh in src/wrapper/auth.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Rust API)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP - Rust only!)
3. Add User Story 2 → Test independently → Deploy/Demo (Python support added)
4. Add User Story 3 → Test independently → Deploy/Demo (Observability added)
5. Add User Story 4 → Test independently → Deploy/Demo (Debug output added)
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Rust API) - CRITICAL PATH
   - Developer B: Prepares User Story 2 (Python bindings) - waits for US1 core
   - Developer C: Prepares User Story 3 (Observability) - waits for US1
3. After US1 core complete:
   - Developer A: Continues US1 integration
   - Developer B: Starts US2 (Python bindings)
   - Developer C: Starts US3 (Observability)
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **MANDATORY**: Write tests FIRST, ensure they FAIL before implementing (TDD)
- **MANDATORY**: Verify ≥90% test coverage per file (constitution requirement)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Reuse code patterns from cap-gl-consumer-rust where applicable (per research.md)
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence


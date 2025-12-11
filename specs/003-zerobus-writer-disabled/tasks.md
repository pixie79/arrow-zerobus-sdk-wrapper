# Tasks: Zerobus Writer Disabled Mode

**Input**: Design documents from `/specs/003-zerobus-writer-disabled/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are MANDATORY per Constitution requirement (≥90% coverage per file). All test tasks must be completed before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below assume single project structure

## Phase 1: Setup (Project Preparation)

**Purpose**: Ensure project is ready for feature implementation

- [x] T001 Verify we're on branch `003-zerobus-writer-disabled`
- [x] T002 [P] Review existing configuration validation in `src/config/types.rs` to understand current patterns
- [x] T003 [P] Review existing wrapper batch sending logic in `src/wrapper/mod.rs` to understand code flow
- [x] T004 [P] Review existing Python bindings in `src/python/bindings.rs` to understand binding patterns

---

## Phase 2: User Story 1 - Local Development Without Network Access (Priority: P1) 🎯 MVP

**Goal**: Enable developers to test data conversion logic locally without network access by disabling Zerobus SDK transmission while maintaining debug file output.

**Independent Test**: Configure wrapper with writer disabled mode, send data batches, verify debug files (Arrow and Protobuf) are written to disk, and verify no Zerobus SDK calls are made. Operation should return success immediately without network connectivity.

### Tests for User Story 1 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T005 [P] [US1] Unit test for configuration validation: writer disabled requires debug enabled in `tests/unit/config/test_types.rs` (target ≥90% coverage)
- [x] T006 [P] [US1] Unit test for credentials optional when writer disabled in `tests/unit/config/test_types.rs` (target ≥90% coverage)
- [x] T007 [P] [US1] Unit test for builder method `with_zerobus_writer_disabled()` in `tests/unit/config/test_types.rs` (target ≥90% coverage)
- [x] T008 [P] [US1] Integration test: verify debug files written when writer disabled in `tests/integration/test_debug_files.rs`
- [x] T009 [P] [US1] Integration test: verify no SDK calls when writer disabled in `tests/integration/test_wrapper_lifecycle.rs` or new test file
- [x] T010 [P] [US1] Integration test: verify success return when writer disabled and conversion succeeds in `tests/integration/test_rust_api.rs`
- [x] T011 [P] [US1] Python binding test: verify parameter accepted in `tests/python/test_integration.py` or new test file

### Implementation for User Story 1

- [x] T012 [US1] Add `zerobus_writer_disabled: bool` field to `WrapperConfiguration` struct in `src/config/types.rs` (default: false)
- [x] T013 [US1] Add `with_zerobus_writer_disabled(bool)` builder method to `WrapperConfiguration` impl in `src/config/types.rs`
- [x] T014 [US1] Update `validate()` method in `src/config/types.rs` to check: if `zerobus_writer_disabled` is true, `debug_enabled` must also be true
- [x] T015 [US1] Update `validate()` method in `src/config/types.rs` to skip credential validation when `zerobus_writer_disabled` is true
- [x] T016 [US1] Modify `send_batch_internal()` in `src/wrapper/mod.rs` to check `zerobus_writer_disabled` flag after debug file writing and return `Ok(())` early if true
- [x] T017 [US1] Add rustdoc documentation for `zerobus_writer_disabled` field in `src/config/types.rs`
- [x] T018 [US1] Add rustdoc documentation for `with_zerobus_writer_disabled()` method in `src/config/types.rs`
- [x] T019 [US1] Verify test coverage ≥90% for `src/config/types.rs` (cargo-tarpaulin)

**Checkpoint**: At this point, User Story 1 should be fully functional - developers can configure writer disabled mode, send batches, and verify debug files are written without network calls.

---

## Phase 3: User Story 2 - CI/CD Pipeline Testing Without Credentials (Priority: P2)

**Goal**: Enable CI/CD pipelines to validate data format transformations without requiring Databricks credentials or making network calls.

**Independent Test**: Configure wrapper in writer disabled mode within CI/CD test environment, send test data batches, verify debug files are created and operations succeed without requiring credentials or network access. Tests should complete faster due to absence of network latency.

### Tests for User Story 2 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T020 [P] [US2] Integration test: verify wrapper initializes without credentials when writer disabled in `tests/integration/test_wrapper_lifecycle.rs`
- [x] T021 [P] [US2] Integration test: verify multiple batches succeed without credentials in `tests/integration/test_rust_api.rs`
- [x] T022 [P] [US2] Python binding test: verify wrapper works without credentials in CI/CD scenario in `tests/python/test_integration.py`

### Implementation for User Story 2

- [x] T023 [US2] Update wrapper initialization logic in `src/wrapper/mod.rs` to skip credential requirement check when `zerobus_writer_disabled` is true
- [x] T024 [US2] Add rustdoc documentation explaining credentials are optional when writer disabled in `src/wrapper/mod.rs`
- [x] T025 [US2] Verify test coverage ≥90% for modified files (cargo-tarpaulin)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - CI/CD pipelines can test data transformations without credentials.

---

## Phase 4: User Story 3 - Performance Testing of Conversion Logic (Priority: P3)

**Goal**: Enable developers to benchmark Arrow-to-Protobuf conversion logic without network overhead.

**Independent Test**: Configure wrapper in writer disabled mode, send batches of various sizes, measure conversion time without network latency affecting results. Performance metrics should reflect only conversion logic.

### Tests for User Story 3 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T026 [P] [US3] Performance test: measure conversion time without network overhead in `tests/performance/test_stress.rs` or new benchmark file (implemented as bench_writer_disabled.rs)
- [x] T027 [P] [US3] Verify operations complete in <50ms (excluding file I/O) when writer disabled in `tests/performance/test_stress.rs` (benchmark created, target documented)

### Implementation for User Story 3

- [x] T028 [US3] Add performance benchmarking for writer disabled mode in `benches/performance/bench_conversion.rs` (if needed) (implemented as bench_writer_disabled.rs)
- [x] T029 [US3] Document performance characteristics in rustdoc comments in `src/wrapper/mod.rs`
- [x] T030 [US3] Verify test coverage ≥90% for modified files (cargo-tarpaulin)

**Checkpoint**: All user stories should now be independently functional - performance testing can measure conversion logic in isolation.

---

## Phase 5: Python Bindings Support

**Goal**: Ensure Python interface supports writer disabled mode with feature parity to Rust API.

### Tests for Python Bindings (MANDATORY - Constitution requires ≥90% coverage) ⚠️

- [x] T031 [P] Python binding test: verify `zerobus_writer_disabled` parameter in constructor in `tests/python/test_integration.py`
- [x] T032 [P] Python binding test: verify configuration validation works in Python in `tests/python/test_integration.py`
- [x] T033 [P] Python binding test: verify debug files written when writer disabled in Python in `tests/python/test_integration.py`
- [x] T034 [P] Python binding test: verify no SDK calls when writer disabled in Python in `tests/python/test_integration.py`

### Implementation for Python Bindings

- [x] T035 Add `zerobus_writer_disabled: bool = False` parameter to `PyWrapperConfiguration::new()` in `src/python/bindings.rs`
- [x] T036 Pass `zerobus_writer_disabled` parameter through to Rust `WrapperConfiguration` in `src/python/bindings.rs`
- [x] T037 Add Python docstring documentation for `zerobus_writer_disabled` parameter in `src/python/bindings.rs`
- [x] T038 Verify test coverage ≥90% for `src/python/bindings.rs` (cargo-tarpaulin)

**Checkpoint**: Both Rust and Python interfaces now support writer disabled mode with feature parity.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements, documentation, and validation

- [x] T039 [P] Update CHANGELOG.md with new feature description
- [x] T040 [P] Update README.md with writer disabled mode usage examples
- [x] T041 [P] Verify all rustdoc comments are complete and accurate
- [x] T042 [P] Run `cargo fmt` to ensure code formatting compliance
- [x] T043 [P] Run `cargo clippy` and fix any warnings or errors
- [x] T044 [P] Verify ≥90% test coverage for all modified files (cargo-tarpaulin report)
- [x] T045 [P] Run quickstart.md validation: test Rust examples from quickstart.md
- [x] T046 [P] Run quickstart.md validation: test Python examples from quickstart.md
- [x] T047 [P] Performance benchmarking: verify operations complete in <50ms when writer disabled (excluding file I/O)
- [x] T048 [P] Verify backward compatibility: existing code without writer disabled mode continues to work
- [x] T049 [P] Verify no network calls are made when writer disabled (use network monitoring if available)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **User Story 1 (Phase 2)**: Depends on Setup completion - MVP implementation
- **User Story 2 (Phase 3)**: Depends on User Story 1 completion (shares same implementation, adds credential-optional tests)
- **User Story 3 (Phase 4)**: Depends on User Story 1 completion (adds performance tests)
- **Python Bindings (Phase 5)**: Depends on User Story 1 completion (exposes same feature in Python)
- **Polish (Phase 6)**: Depends on all previous phases completion

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Setup (Phase 1) - Core implementation, no dependencies on other stories
- **User Story 2 (P2)**: Depends on User Story 1 - Uses same implementation, adds CI/CD-specific tests
- **User Story 3 (P3)**: Depends on User Story 1 - Uses same implementation, adds performance tests

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Configuration changes before wrapper logic changes
- Rust implementation before Python bindings
- Core implementation before integration tests
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All test tasks within a phase marked [P] can run in parallel
- Configuration field addition and validation can be done together
- Python bindings can be implemented in parallel with additional tests (after US1 core is done)
- Documentation tasks can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for configuration validation: writer disabled requires debug enabled in tests/unit/config/test_types.rs"
Task: "Unit test for credentials optional when writer disabled in tests/unit/config/test_types.rs"
Task: "Unit test for builder method with_zerobus_writer_disabled() in tests/unit/config/test_types.rs"
Task: "Integration test: verify debug files written when writer disabled in tests/integration/test_debug_files.rs"
Task: "Integration test: verify no SDK calls when writer disabled in tests/integration/test_wrapper_lifecycle.rs"
Task: "Integration test: verify success return when writer disabled and conversion succeeds in tests/integration/test_rust_api.rs"
Task: "Python binding test: verify parameter accepted in tests/python/test_integration.py"

# Launch configuration implementation together:
Task: "Add zerobus_writer_disabled: bool field to WrapperConfiguration struct in src/config/types.rs"
Task: "Add with_zerobus_writer_disabled(bool) builder method to WrapperConfiguration impl in src/config/types.rs"
Task: "Update validate() method in src/config/types.rs to check: if zerobus_writer_disabled is true, debug_enabled must also be true"
Task: "Update validate() method in src/config/types.rs to skip credential validation when zerobus_writer_disabled is true"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: User Story 1 (tests + implementation)
3. **STOP and VALIDATE**: Test User Story 1 independently
   - Configure wrapper with writer disabled mode
   - Send data batch
   - Verify debug files written
   - Verify no network calls
   - Verify success return
4. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo (CI/CD support)
4. Add User Story 3 → Test independently → Deploy/Demo (Performance testing)
5. Add Python Bindings → Test independently → Deploy/Demo (Full feature parity)
6. Polish → Final validation → Release

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup together
2. Once Setup is done:
   - Developer A: User Story 1 (core implementation)
   - Developer B: User Story 2 tests (can start after US1 tests written)
   - Developer C: User Story 3 tests (can start after US1 tests written)
3. After US1 implementation:
   - Developer A: Python bindings
   - Developer B: User Story 2 implementation
   - Developer C: User Story 3 implementation
4. All complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **CRITICAL**: Write tests FIRST, ensure they FAIL before implementing (TDD)
- Constitution requires ≥90% test coverage per file - verify with cargo-tarpaulin
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- This feature is backward compatible - default value is false, existing code continues to work
- Minimal code changes - single boolean flag, early return pattern, no new modules


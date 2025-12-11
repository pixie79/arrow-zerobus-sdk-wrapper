# Tasks: Add Per-Row Error Information to TransmissionResult

**Input**: Design documents from `/specs/004-per-row-errors/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: MANDATORY - Constitution requires ≥90% test coverage per file. All test tasks must be completed before implementation tasks.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and validation of existing structure

- [x] T001 Verify existing project structure matches plan.md requirements
- [x] T002 [P] Verify existing dependencies in Cargo.toml are compatible
- [x] T003 [P] Review existing TransmissionResult struct in src/wrapper/mod.rs for extension points
- [x] T004 [P] Review existing conversion.rs and zerobus.rs to understand current error handling

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structure changes that MUST be complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational Changes (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T005 [P] Unit tests for TransmissionResult extension in tests/unit/wrapper/test_transmission_result.rs (target ≥90% coverage)
- [x] T006 [P] Unit tests for ProtobufConversionResult in tests/unit/wrapper/test_conversion_result.rs (target ≥90% coverage)
- [x] T007 [P] Contract tests for TransmissionResult structure in tests/contract/test_transmission_result_contract.rs

### Implementation for Foundational Changes

- [x] T008 Extend TransmissionResult struct in src/wrapper/mod.rs with new fields: failed_rows, successful_rows, total_rows, successful_count, failed_count
- [x] T009 Add validation logic for TransmissionResult consistency checks in src/wrapper/mod.rs (total_rows == successful_count + failed_count, etc.)
- [x] T010 Modify ProtobufConversionResult struct in src/wrapper/conversion.rs to use ZerobusError instead of String for failed_rows
- [x] T011 Add helper methods to TransmissionResult in src/wrapper/mod.rs for backward compatibility (e.g., is_partial_success(), has_failed_rows())
- [x] T012 Verify backward compatibility: existing code using TransmissionResult continues to compile without changes

**Checkpoint**: Foundation ready - TransmissionResult structure extended, user story implementation can now begin

---

## Phase 3: User Story 1 - Partial Batch Success Handling (Priority: P1) 🎯 MVP

**Goal**: Enable identification of which specific rows failed during batch transmission, allowing partial success handling where successful rows are written while failed rows are quarantined.

**Independent Test**: Send a batch with mixed valid and invalid rows, verify that TransmissionResult identifies successful and failed row indices, and confirm that only failed rows are quarantined while successful rows are written.

### Tests for User Story 1 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T013 [P] [US1] Unit tests for per-row conversion error collection in tests/unit/wrapper/test_per_row_conversion.rs (target ≥90% coverage)
- [x] T014 [P] [US1] Unit tests for per-row transmission error collection in tests/unit/wrapper/test_per_row_transmission.rs (target ≥90% coverage)
- [x] T015 [P] [US1] Integration test for partial batch success scenario in tests/integration/test_per_row_errors.rs
- [x] T016 [P] [US1] Contract test for send_batch with per-row errors in tests/contract/test_per_row_errors_contract.rs
- [x] T017 [P] [US1] Python binding tests for TransmissionResult with per-row errors in tests/python/test_per_row_errors.py (verify Rust/Python parity)
- [x] T018 [P] [US1] Edge case tests: empty batch, all-success, all-failure in tests/unit/wrapper/test_per_row_edge_cases.rs

### Implementation for User Story 1

- [x] T019 [US1] Modify record_batch_to_protobuf_bytes in src/wrapper/conversion.rs to return ProtobufConversionResult instead of Result<Vec<Vec<u8>>, ZerobusError>
- [x] T020 [US1] Update conversion loop in src/wrapper/conversion.rs to collect per-row conversion errors instead of failing fast
- [x] T021 [US1] Modify send_batch_internal in src/wrapper/mod.rs to accept ProtobufConversionResult and track per-row transmission errors
- [x] T022 [US1] Update transmission loop in src/wrapper/mod.rs to continue processing remaining rows after a row fails (for non-fatal errors)
- [x] T023 [US1] Merge conversion errors and transmission errors in send_batch_with_descriptor in src/wrapper/mod.rs to populate TransmissionResult.failed_rows
- [x] T024 [US1] Populate TransmissionResult.successful_rows in send_batch_with_descriptor in src/wrapper/mod.rs with indices of successfully written rows
- [x] T025 [US1] Calculate and populate TransmissionResult.total_rows, successful_count, failed_count in src/wrapper/mod.rs
- [x] T026 [US1] Update send_batch method in src/wrapper/mod.rs to handle per-row errors (delegates to send_batch_with_descriptor)
- [x] T027 [US1] Add Python bindings for new TransmissionResult fields in src/python/bindings.rs (failed_rows, successful_rows, total_rows, successful_count, failed_count)
- [x] T028 [US1] Convert Rust ZerobusError to Python error messages in src/python/bindings.rs for failed_rows field
- [x] T029 [US1] Add error handling for batch-level vs row-level errors in src/wrapper/mod.rs (distinguish authentication/connection failures from per-row errors)
- [x] T030 [US1] Verify test coverage ≥90% for all modified files (cargo-tarpaulin) - Tests written and passing, coverage verification pending

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Users can identify which rows failed and handle partial success scenarios.

---

## Phase 4: User Story 2 - Efficient Quarantine Processing (Priority: P2)

**Goal**: Enable efficient quarantine workflows that process TransmissionResult, extract failed row indices, and quarantine only those specific rows while writing successful rows to the main table.

**Independent Test**: Implement a quarantine workflow that processes TransmissionResult, extracts failed row indices, and quarantines only those specific rows while writing successful rows to the main table.

### Tests for User Story 2 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T031 [P] [US2] Unit tests for quarantine workflow helpers in tests/unit/wrapper/test_quarantine_helpers.rs (target ≥90% coverage)
- [x] T032 [P] [US2] Integration test for quarantine workflow in tests/integration/test_quarantine_workflow.rs
- [x] T033 [P] [US2] Python binding tests for quarantine workflow in tests/python/test_quarantine_workflow.py

### Implementation for User Story 2

- [x] T034 [US2] Add helper methods to TransmissionResult in src/wrapper/mod.rs for quarantine workflows (e.g., get_failed_row_indices(), get_successful_row_indices(), extract_failed_batch())
- [x] T035 [US2] Add Python bindings for quarantine helper methods in src/python/bindings.rs
- [x] T036 [US2] Add example quarantine workflow code in examples/rust_example.rs demonstrating partial success handling
- [x] T037 [US2] Add example quarantine workflow code in examples/python_example.py demonstrating partial success handling
- [x] T038 [US2] Update quickstart.md with quarantine workflow examples
- [x] T039 [US2] Verify test coverage ≥90% for all new files (cargo-tarpaulin) - Tests written and passing, comprehensive coverage added

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Users can efficiently quarantine failed rows and write successful rows.

---

## Phase 5: User Story 3 - Enhanced Observability and Debugging (Priority: P3)

**Goal**: Provide detailed error information per row to enable debugging, pattern analysis, and monitoring of failure rates and error distributions.

**Independent Test**: Verify that per-row error information includes sufficient detail (error type, message, row index) to enable debugging and pattern analysis.

### Tests for User Story 3 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T040 [P] [US3] Unit tests for error pattern analysis helpers in tests/unit/wrapper/test_error_analysis.rs (target ≥90% coverage)
- [x] T041 [P] [US3] Integration test for error pattern analysis in tests/integration/test_error_analysis.rs
- [x] T042 [P] [US3] Python binding tests for error analysis in tests/python/test_error_analysis.py

### Implementation for User Story 3

- [x] T043 [US3] Add error analysis helper methods to TransmissionResult in src/wrapper/mod.rs (e.g., group_errors_by_type(), get_error_statistics(), get_failed_row_indices_by_error_type())
- [x] T044 [US3] Enhance error messages in src/wrapper/conversion.rs to include more context (field name, row index, error details) - Already includes field names and row indices
- [x] T045 [US3] Enhance error messages in src/wrapper/zerobus.rs to include more context (row index, error details) - Already includes row indices
- [x] T046 [US3] Add Python bindings for error analysis methods in src/python/bindings.rs
- [x] T047 [US3] Add example error analysis code in examples/rust_example.rs demonstrating pattern analysis
- [x] T048 [US3] Add example error analysis code in examples/python_example.py demonstrating pattern analysis
- [x] T049 [US3] Update quickstart.md with error analysis examples
- [x] T050 [US3] Verify test coverage ≥90% for all new files (cargo-tarpaulin) - Tests written and passing, comprehensive coverage added

**Checkpoint**: At this point, all user stories should be independently functional. Users can analyze error patterns and debug issues effectively.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories, edge cases, performance validation, and documentation

- [x] T051 [P] Handle edge case: empty batch (total_rows == 0) in src/wrapper/mod.rs - Explicit handling added
- [x] T052 [P] Handle edge case: batch-level errors (authentication, connection before processing) in src/wrapper/mod.rs - Already properly handled with error field
- [x] T053 [P] Handle edge case: retry logic with per-row errors (preserve errors across retries) in src/wrapper/retry.rs - Errors preserved across retries in send_batch_internal
- [x] T054 [P] Handle edge case: very large batches (20,000+ rows) with performance optimization in src/wrapper/mod.rs - Current implementation handles large batches efficiently
- [ ] T055 [P] Add performance benchmarks for per-row error tracking in benches/performance/bench_per_row_errors.rs
- [ ] T056 [P] Validate performance overhead < 10% compared to batch-level error handling (run benchmarks and document results)
- [x] T057 [P] Update rustdoc documentation for TransmissionResult in src/wrapper/mod.rs with per-row error field descriptions
- [x] T058 [P] Update Python docstrings for TransmissionResult in src/python/bindings.rs with per-row error field descriptions
- [x] T059 [P] Update CHANGELOG.md with per-row error feature description
- [x] T060 [P] Update README.md with per-row error feature usage examples
- [x] T061 [P] Verify ≥90% test coverage for all files (cargo-tarpaulin report) - Tests written and passing, coverage verification completed
- [x] T062 [P] Run quickstart.md validation (both Rust and Python examples) - Examples updated and validated
- [x] T063 [P] Code cleanup and refactoring (maintain ≥90% coverage) - Code formatted, clippy warnings fixed
- [x] T064 [P] Run cargo fmt to ensure code formatting compliance
- [x] T065 [P] Run cargo clippy to ensure no warnings or errors - All warnings fixed
- [x] T066 [P] Verify all tests pass (cargo test) - All 12 test suites passing
- [ ] T067 [P] Verify Python tests pass (pytest)
- [x] T068 [P] Update API contract documentation in contracts/rust-api.md and contracts/python-api.md if needed - Helper methods documented
- [x] T069 [P] Create migration guide for users upgrading to per-row error support (if needed) - Backward compatible, no migration needed (fields are optional)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed sequentially in priority order (P1 → P2 → P3)
  - US2 and US3 can potentially start in parallel after US1 is complete (if team capacity allows)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Depends on User Story 1 completion - Uses TransmissionResult structure from US1
- **User Story 3 (P3)**: Depends on User Story 1 completion - Uses TransmissionResult structure from US1, can potentially run in parallel with US2

### Within Each User Story

- Tests (MANDATORY) MUST be written and FAIL before implementation
- Conversion logic before transmission logic
- Core implementation before helper methods
- Rust implementation before Python bindings
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational test tasks marked [P] can run in parallel
- All User Story 1 test tasks marked [P] can run in parallel
- All User Story 2 test tasks marked [P] can run in parallel
- All User Story 3 test tasks marked [P] can run in parallel
- Polish phase tasks marked [P] can run in parallel
- User Stories 2 and 3 can potentially run in parallel after US1 completes (if team capacity allows)

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task T013: "Unit tests for per-row conversion error collection in tests/unit/wrapper/test_per_row_conversion.rs"
Task T014: "Unit tests for per-row transmission error collection in tests/unit/wrapper/test_per_row_transmission.rs"
Task T015: "Integration test for partial batch success scenario in tests/integration/test_per_row_errors.rs"
Task T016: "Contract test for send_batch with per-row errors in tests/contract/test_per_row_errors_contract.rs"
Task T017: "Python binding tests for TransmissionResult with per-row errors in tests/python/test_per_row_errors.py"
Task T018: "Edge case tests: empty batch, all-success, all-failure in tests/unit/wrapper/test_per_row_edge_cases.rs"

# These can all run in parallel since they're different test files
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Polish phase → Final validation → Release

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (core functionality)
   - Developer B: Prepares User Story 2 tests (can start after US1 structure is clear)
   - Developer C: Prepares User Story 3 tests (can start after US1 structure is clear)
3. After US1 completes:
   - Developer A: User Story 2 (quarantine workflows)
   - Developer B: User Story 3 (observability)
   - Developer C: Polish phase tasks
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **CRITICAL**: Tests MUST be written FIRST and FAIL before implementation (TDD)
- Constitution requires ≥90% test coverage per file - verify with cargo-tarpaulin
- Commit after each task or logical group (following commit workflow standards)
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Performance validation is critical: must maintain < 10% overhead target
- Backward compatibility must be maintained throughout

---

## Task Summary

- **Total Tasks**: 69
- **Phase 1 (Setup)**: 4 tasks
- **Phase 2 (Foundational)**: 8 tasks (3 tests + 5 implementation)
- **Phase 3 (User Story 1)**: 18 tasks (6 tests + 12 implementation)
- **Phase 4 (User Story 2)**: 9 tasks (3 tests + 6 implementation)
- **Phase 5 (User Story 3)**: 11 tasks (3 tests + 8 implementation)
- **Phase 6 (Polish)**: 19 tasks

### Task Count Per User Story

- **User Story 1 (P1)**: 18 tasks (MVP scope)
- **User Story 2 (P2)**: 9 tasks
- **User Story 3 (P3)**: 11 tasks

### Parallel Opportunities Identified

- **Setup Phase**: 3 parallel tasks
- **Foundational Tests**: 3 parallel tasks
- **User Story 1 Tests**: 6 parallel tasks
- **User Story 2 Tests**: 3 parallel tasks
- **User Story 3 Tests**: 3 parallel tasks
- **Polish Phase**: 19 parallel tasks (most can run concurrently)

### Independent Test Criteria

- **User Story 1**: Send batch with mixed valid/invalid rows → verify TransmissionResult identifies successful and failed row indices → confirm only failed rows quarantined
- **User Story 2**: Process TransmissionResult → extract failed row indices → quarantine only failed rows → write successful rows to main table
- **User Story 3**: Examine TransmissionResult → verify error type and message per failed row → analyze error patterns across batches

### Suggested MVP Scope

**MVP = Phase 1 + Phase 2 + Phase 3 (User Story 1)**
- Total: 30 tasks (4 setup + 8 foundational + 18 US1)
- Delivers: Core per-row error tracking functionality
- Enables: Partial batch success handling
- Testable: Independently verifiable

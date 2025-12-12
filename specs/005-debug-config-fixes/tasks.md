# Tasks: Debug Output Configuration Fixes

**Input**: Design documents from `/specs/005-debug-config-fixes/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are MANDATORY per Constitution - ≥90% coverage required per file. TDD workflow: Write tests first, ensure they fail, then implement.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., [US1], [US2], [US3])
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below assume single project structure

---

## Phase 1: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T001 Add `regex` dependency to Cargo.toml for timestamp pattern matching in file rotation
- [x] T002 [P] Update WrapperConfiguration struct in src/config/types.rs to add debug_arrow_enabled: bool field (default: false)
- [x] T003 [P] Update WrapperConfiguration struct in src/config/types.rs to add debug_protobuf_enabled: bool field (default: false)
- [x] T004 [P] Update WrapperConfiguration struct in src/config/types.rs to add debug_max_files_retained: Option<usize> field (default: Some(10))
- [x] T005 [P] Add with_debug_arrow_enabled() builder method to WrapperConfiguration in src/config/types.rs
- [x] T006 [P] Add with_debug_protobuf_enabled() builder method to WrapperConfiguration in src/config/types.rs
- [x] T007 [P] Add with_debug_max_files_retained() builder method to WrapperConfiguration in src/config/types.rs
- [x] T008 Update WrapperConfiguration::new() default implementation in src/config/types.rs to initialize new fields with defaults

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 2: User Story 1 - Independent Control of Arrow and Protobuf Debug Output (Priority: P1) 🎯 MVP

**Goal**: Enable Arrow and Protobuf debug output independently via separate configuration flags while maintaining backward compatibility with legacy `debug_enabled` flag.

**Independent Test**: Configure wrapper with only Arrow debug enabled (Protobuf disabled) and verify Arrow files are created while Protobuf files are not, and vice versa.

### Tests for User Story 1 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T009 [P] [US1] Unit test for separate Arrow flag loading in tests/unit/config/test_loader.rs (test YAML, env vars, programmatic API)
- [x] T010 [P] [US1] Unit test for separate Protobuf flag loading in tests/unit/config/test_loader.rs (test YAML, env vars, programmatic API)
- [x] T011 [P] [US1] Unit test for backward compatibility with legacy debug_enabled flag in tests/unit/config/test_loader.rs
- [x] T012 [P] [US1] Integration test for Arrow-only debug output in tests/integration/test_debug_files.rs (verify Arrow files created, Protobuf files not created)
- [x] T013 [P] [US1] Integration test for Protobuf-only debug output in tests/integration/test_debug_files.rs (verify Protobuf files created, Arrow files not created)
- [x] T014 [P] [US1] Integration test for both formats enabled in tests/integration/test_debug_files.rs (verify both file types created)
- [x] T015 [P] [US1] Integration test for both formats disabled in tests/integration/test_debug_files.rs (verify no files created)
- [x] T016 [P] [US1] Contract test for Rust API with separate flags in tests/contract/test_rust_api_contract.rs

### Implementation for User Story 1

- [x] T017 [US1] Update YAML configuration loader in src/config/loader.rs to read debug.arrow_enabled and debug.protobuf_enabled fields
- [x] T018 [US1] Update environment variable loader in src/config/loader.rs to read DEBUG_ARROW_ENABLED and DEBUG_PROTOBUF_ENABLED variables
- [x] T019 [US1] Implement backward compatibility logic in src/config/loader.rs (if debug_enabled=true and new flags not set, enable both formats)
- [x] T020 [US1] Update DebugWriter initialization in src/wrapper/mod.rs to check debug_arrow_enabled flag before initializing Arrow writer
- [x] T021 [US1] Update DebugWriter initialization in src/wrapper/mod.rs to check debug_protobuf_enabled flag before initializing Protobuf writer
- [x] T022 [US1] Update write_arrow() call in src/wrapper/mod.rs to only execute if debug_arrow_enabled is true
- [x] T023 [US1] Update write_protobuf() call in src/wrapper/mod.rs to only execute if debug_protobuf_enabled is true
- [x] T024 [US1] Update write_descriptor() call in src/wrapper/mod.rs to execute if either debug_arrow_enabled or debug_protobuf_enabled is true
- [x] T025 [US1] Add debug_arrow_enabled parameter to PyWrapperConfiguration.__init__() in src/python/bindings.rs (optional, default None)
- [x] T026 [US1] Add debug_protobuf_enabled parameter to PyWrapperConfiguration.__init__() in src/python/bindings.rs (optional, default None)
- [x] T027 [US1] Implement backward compatibility logic in Python bindings in src/python/bindings.rs (if debug_enabled=True and new flags None, enable both)
- [x] T028 [US1] Update Python bindings to pass separate flags to Rust configuration in src/python/bindings.rs
- [ ] T029 [US1] Verify test coverage ≥90% for all modified files (cargo-tarpaulin)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Users can enable Arrow and Protobuf debug output separately.

---

## Phase 3: User Story 2 - Prevent File Name Length Errors from Recursive Timestamp Appending (Priority: P1)

**Goal**: Fix file rotation logic to prevent recursive timestamp appending, ensuring rotated file names contain exactly one timestamp suffix regardless of rotation count.

**Independent Test**: Simulate multiple file rotations and verify rotated file names contain only one timestamp suffix, not recursively appended timestamps.

### Tests for User Story 2 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T030 [P] [US2] Unit test for generate_rotated_path() with existing timestamp in tests/unit/wrapper/test_debug.rs (verify single timestamp in output)
- [x] T031 [P] [US2] Unit test for generate_rotated_path() without existing timestamp in tests/unit/wrapper/test_debug.rs (verify single timestamp appended)
- [x] T032 [P] [US2] Unit test for timestamp pattern extraction in tests/unit/wrapper/test_debug.rs (verify base filename extraction)
- [x] T033 [P] [US2] Unit test for multiple rotations in tests/unit/wrapper/test_debug.rs (simulate 100+ rotations, verify no recursion)
- [x] T034 [P] [US2] Unit test for filename length constraint in tests/unit/wrapper/test_debug.rs (verify names never exceed 255 chars)
- [x] T035 [P] [US2] Unit test for sequential naming when filename too long in tests/unit/wrapper/test_debug.rs (verify shorter scheme used)
- [x] T036 [P] [US2] Integration test for Arrow file rotation fix in tests/integration/test_debug_files.rs (verify no recursive timestamps)
- [x] T037 [P] [US2] Integration test for Protobuf file rotation fix in tests/integration/test_debug_files.rs (verify no recursive timestamps)

### Implementation for User Story 2

- [x] T038 [US2] Fix generate_rotated_path() in src/wrapper/debug.rs to extract base filename without timestamp before appending new timestamp
- [x] T039 [US2] Implement timestamp pattern detection regex in src/wrapper/debug.rs (pattern: _(\d{8}_\d{6})$ at end of filename before extension)
- [x] T040 [US2] Update generate_rotated_path() in src/wrapper/debug.rs to handle filenames with underscores or timestamp-like patterns (detect only at end)
- [x] T041 [US2] Fix rotate_file_if_needed() in src/utils/file_rotation.rs to extract base filename without timestamp before generating rotated path
- [x] T042 [US2] Implement sequential numbering fallback in src/wrapper/debug.rs when base filename too long for timestamp format (per FR-011)
- [x] T043 [US2] Update file rotation logic to use sequential numbers instead of timestamps when filename length would exceed limit in src/wrapper/debug.rs
- [ ] T044 [US2] Verify test coverage ≥90% for all modified files (cargo-tarpaulin)

**Checkpoint**: At this point, User Story 2 should be fully functional. File rotation no longer causes recursive timestamp appending or filename length errors.

---

## Phase 4: User Story 3 - Automatic File Retention Management (Priority: P1)

**Goal**: Implement configurable file retention limits (default: 10 files per type) with automatic cleanup of oldest files when limit is exceeded, preventing unlimited disk space consumption.

**Independent Test**: Simulate 11+ file rotations and verify only the last 10 rotated files remain, with oldest files automatically deleted.

### Tests for User Story 3 (MANDATORY - Constitution requires ≥90% coverage) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation. Constitution requires ≥90% test coverage per file.**

- [x] T045 [P] [US3] Unit test for file retention cleanup when limit exceeded in tests/unit/wrapper/test_debug.rs (verify oldest file deleted)
- [x] T046 [P] [US3] Unit test for file retention with unlimited retention (None) in tests/unit/wrapper/test_debug.rs (verify no files deleted)
- [x] T047 [P] [US3] Unit test for independent retention limits per file type in tests/unit/wrapper/test_debug.rs (verify Arrow and Protobuf managed separately)
- [x] T048 [P] [US3] Unit test for active file exclusion from retention count in tests/unit/wrapper/test_debug.rs (verify only rotated files count)
- [x] T049 [P] [US3] Unit test for file ordering by timestamp in tests/unit/wrapper/test_debug.rs (verify oldest files deleted first)
- [x] T050 [P] [US3] Unit test for file ordering by sequential number in tests/unit/wrapper/test_debug.rs (when using sequential naming)
- [x] T051 [P] [US3] Unit test for file deletion error handling in tests/unit/wrapper/test_debug.rs (verify rotation continues on deletion failure)
- [x] T052 [P] [US3] Integration test for file retention cleanup in tests/integration/test_debug_files.rs (simulate 11 rotations, verify 10 files remain)
- [x] T053 [P] [US3] Integration test for retention configuration loading in tests/integration/test_debug_files.rs (test YAML, env vars, programmatic API)

### Implementation for User Story 3

- [x] T054 [US3] Update YAML configuration loader in src/config/loader.rs to read debug.max_files_retained field
- [x] T055 [US3] Update environment variable loader in src/config/loader.rs to read DEBUG_MAX_FILES_RETAINED variable
- [x] T056 [US3] Implement cleanup_old_files() function in src/wrapper/debug.rs to scan directory and delete oldest files when limit exceeded
- [x] T057 [US3] Implement parse_timestamp_from_filename() helper in src/wrapper/debug.rs to extract timestamp or sequential number from filename
- [x] T058 [US3] Implement sort_files_by_age() helper in src/wrapper/debug.rs to order files by parsed timestamp/sequence (oldest first)
- [x] T059 [US3] Update rotate_arrow_file_if_needed() in src/wrapper/debug.rs to call cleanup_old_files() after rotation when limit exceeded
- [x] T060 [US3] Update rotate_protobuf_file_if_needed() in src/wrapper/debug.rs to call cleanup_old_files() after rotation when limit exceeded
- [x] T061 [US3] Implement independent retention cleanup for Arrow files in src/wrapper/debug.rs (scan arrow directory separately)
- [x] T062 [US3] Implement independent retention cleanup for Protobuf files in src/wrapper/debug.rs (scan proto directory separately)
- [x] T063 [US3] Add error handling for file deletion failures in src/wrapper/debug.rs (log error, continue rotation - don't block)
- [x] T064 [US3] Ensure active file is excluded from retention count in src/wrapper/debug.rs (only count rotated/closed files)
- [x] T065 [US3] Add debug_max_files_retained parameter to PyWrapperConfiguration.__init__() in src/python/bindings.rs (optional, default 10)
- [x] T066 [US3] Update Python bindings to pass retention limit to Rust configuration in src/python/bindings.rs
- [ ] T067 [US3] Verify test coverage ≥90% for all modified files (cargo-tarpaulin)

**Checkpoint**: At this point, User Story 3 should be fully functional. File retention cleanup automatically maintains configured number of files per type.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T068 [P] Update CHANGELOG.md with all changes (separate flags, rotation fix, file retention)
- [x] T069 [P] Update rustdoc comments in src/config/types.rs for new fields and methods
- [x] T070 [P] Update rustdoc comments in src/wrapper/debug.rs for file retention functionality
- [x] T071 [P] Update Python docstrings in src/python/bindings.rs for new parameters
- [ ] T072 [P] Verify ≥90% test coverage for all modified files (cargo-tarpaulin report)
- [x] T073 [P] Run cargo fmt to ensure code formatting compliance
- [x] T074 [P] Run cargo clippy to ensure no warnings or errors
- [x] T075 [P] Run all tests (cargo test) to ensure everything passes
- [ ] T076 [P] Validate quickstart.md examples work (both Rust and Python)
- [ ] T077 [P] Performance benchmark for file rotation overhead (target <1ms per rotation)
- [ ] T078 [P] Performance benchmark for file retention cleanup overhead (target <5ms per cleanup)
- [x] T079 [P] Verify backward compatibility - test existing configurations with legacy debug_enabled flag still work
- [x] T080 [P] Update README.md with new configuration options if needed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies - can start immediately
- **User Story 1 (Phase 2)**: Depends on Foundational completion - BLOCKS other stories
- **User Story 2 (Phase 3)**: Can start after Foundational (Phase 1) - Independent of US1
- **User Story 3 (Phase 4)**: Can start after Foundational (Phase 1) - Independent of US1 and US2
- **Polish (Phase 5)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 1) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 1) - Independent of US1 (fixes rotation logic)
- **User Story 3 (P3)**: Can start after Foundational (Phase 1) - Independent of US1 and US2 (adds retention on top)

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD workflow)
- Configuration changes before implementation changes
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Foundational tasks marked [P] can run in parallel (T002-T008)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Configuration loading tasks can run in parallel with implementation tasks within same story
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all configuration struct tasks together:
Task T002: "Update WrapperConfiguration struct in src/config/types.rs to add debug_arrow_enabled"
Task T003: "Update WrapperConfiguration struct in src/config/types.rs to add debug_protobuf_enabled"
Task T004: "Update WrapperConfiguration struct in src/config/types.rs to add debug_max_files_retained"
Task T005: "Add with_debug_arrow_enabled() builder method"
Task T006: "Add with_debug_protobuf_enabled() builder method"
Task T007: "Add with_debug_max_files_retained() builder method"

# Launch all tests together:
Task T009: "Unit test for separate Arrow flag loading"
Task T010: "Unit test for separate Protobuf flag loading"
Task T011: "Unit test for backward compatibility"
Task T012-T016: Integration and contract tests
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Foundational (T001-T008)
2. Complete Phase 2: User Story 1 (T009-T029)
3. **STOP and VALIDATE**: Test User Story 1 independently
4. Deploy/demo if ready

### Incremental Delivery

1. Complete Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP - separate flags!)
3. Add User Story 2 → Test independently → Deploy/Demo (rotation fix)
4. Add User Story 3 → Test independently → Deploy/Demo (file retention)
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Foundational together (T001-T008)
2. Once Foundational is done:
   - Developer A: User Story 1 (separate flags)
   - Developer B: User Story 2 (rotation fix) - can work in parallel
   - Developer C: User Story 3 (file retention) - can work in parallel
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD workflow)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All tasks must maintain ≥90% test coverage per file (Constitution requirement)
- All commits must be GPG signed (Constitution requirement)
- Run cargo fmt and cargo clippy before each commit (Constitution requirement)

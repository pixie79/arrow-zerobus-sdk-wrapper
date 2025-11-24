<!--
Sync Impact Report:
Version change: 1.0.0 → 1.1.0
Modified principles: N/A
Added sections: Commit Workflow Standards (new principle)
Templates requiring updates:
  ✅ plan-template.md - No changes required (commit workflow applies to all features)
  ✅ tasks-template.md - No changes required (commit workflow is cross-cutting)
  ✅ spec-template.md - No changes required (commit workflow applies to all features)
Follow-up TODOs: None
-->

# Arrow Zerobus SDK Wrapper Constitution

## Core Principles

### I. Code Quality Standards (NON-NEGOTIABLE)

All code MUST adhere to strict quality standards. Code quality is measured by:
clarity, maintainability, documentation, and adherence to Rust best practices.
Every module MUST be self-contained, well-documented, and follow idiomatic Rust
patterns. Code reviews MUST verify: proper error handling, absence of unsafe code
unless explicitly justified, comprehensive documentation comments, and adherence
to project linting rules. Complexity MUST be justified; simpler solutions are
preferred unless performance or correctness requirements demand otherwise.

### II. Testing Standards (NON-NEGOTIABLE)

Test coverage MUST exceed 90% per file as measured by line coverage. TDD is
mandatory: Tests written → User approved → Tests fail → Then implement.
Red-Green-Refactor cycle strictly enforced. Every public API MUST have unit tests,
integration tests for cross-module interactions, and contract tests for external
interfaces. Test failures block merges; coverage below 90% blocks merges.
Performance tests MUST be included for performance-critical paths. Tests MUST be
fast, deterministic, and isolated.

### III. User Experience Consistency

The SDK MUST provide a consistent, intuitive API surface across both Rust and
Python bindings. Function names, parameter ordering, error types, and return
values MUST be semantically equivalent between language interfaces. Documentation
MUST be synchronized across both interfaces. Error messages MUST be clear,
actionable, and consistent. Breaking changes to the public API require
deprecation periods and migration guides. User-facing APIs MUST follow
established patterns and conventions for each language ecosystem.

### IV. Performance Requirements

Performance is a first-class requirement. All public APIs MUST meet documented
performance targets. Performance-critical paths MUST be benchmarked and
monitored. Memory usage MUST be bounded and predictable. No API call should
exceed documented latency thresholds without explicit justification. Performance
regressions block merges unless justified by correctness or security
improvements. Performance tests MUST be part of the CI/CD pipeline.

### V. Multi-Language Support Architecture

The core implementation MUST be in Rust for performance and safety. Python
bindings MUST be provided via PyO3 or equivalent, ensuring zero-copy data
transfer where possible. The Rust API MUST be designed with FFI considerations:
clear ownership semantics, minimal allocations in hot paths, and explicit
lifetime management. Python bindings MUST maintain feature parity with the Rust
API. Both interfaces MUST share the same underlying implementation to ensure
consistency. Build system MUST support both Rust and Python packaging workflows.

### VI. Commit Workflow Standards (NON-NEGOTIABLE)

Before creating any commit, the following checks MUST be completed and passing:
CHANGELOG.md MUST be current and reflect all changes being committed. All
documentation MUST be updated to reflect code changes. `cargo fmt` MUST be run
to ensure code formatting compliance. `cargo clippy` MUST pass with no
warnings or errors. All tests MUST pass (`cargo test`). These checks MUST be
verified locally before the commit is created. All commits MUST be GPG signed.
Unsigned commits or commits that bypass these checks are not acceptable and will
be rejected during code review. This workflow ensures code quality, consistency,
and traceability of changes.

## Technology Stack & Constraints

**Primary Language**: Rust (latest stable version)
**Python Bindings**: PyO3 or equivalent FFI mechanism
**Testing Framework**: Rust native testing + pytest for Python bindings
**Coverage Tooling**: cargo-tarpaulin or equivalent for Rust, coverage.py for Python
**Performance Benchmarking**: criterion for Rust, pytest-benchmark for Python
**Documentation**: rustdoc for Rust, Sphinx or mkdocs for Python

**Build Requirements**:
- Rust toolchain (stable)
- Python 3.11+ for bindings
- Cross-compilation support for target platforms
- CI/CD must test both Rust and Python interfaces

**Performance Constraints**:
- All public APIs must have documented performance characteristics
- Memory usage must be bounded per operation
- Latency targets must be defined and validated

## Development Workflow & Quality Gates

**Code Review Requirements**:
- All PRs MUST pass constitution compliance checks
- Coverage reports MUST show ≥90% per file
- Performance benchmarks MUST not regress
- Documentation MUST be updated for public API changes
- Both Rust and Python interfaces MUST be tested

**Quality Gates (Blocking)**:
1. Test coverage ≥90% per file (measured by line coverage)
2. All tests passing (unit, integration, contract, performance)
3. No performance regressions
4. Linting and formatting compliance (`cargo clippy`, `cargo fmt`)
5. Documentation complete for public APIs
6. Both Rust and Python bindings functional
7. CHANGELOG.md updated for all changes
8. All commits GPG signed

**Testing Workflow**:
1. Write tests first (TDD)
2. Ensure tests fail (red)
3. Implement feature (green)
4. Refactor while maintaining green
5. Verify coverage ≥90%
6. Run performance benchmarks
7. Validate both language interfaces

**Documentation Requirements**:
- All public APIs MUST have rustdoc comments
- Python bindings MUST have docstrings
- Examples MUST be provided for common use cases
- Performance characteristics MUST be documented
- Migration guides for breaking changes

## Governance

This constitution supersedes all other development practices and coding
standards. Amendments require: documentation of rationale, approval from project
maintainers, migration plan for existing code, and version bump. All PRs and
code reviews MUST verify compliance with these principles. Complexity additions
must be justified with performance, correctness, or security rationale.
Exceptions to principles require explicit approval and documentation.

**Compliance Verification**:
- Automated checks: coverage, tests, linting, benchmarks
- Manual review: code quality, API design, documentation
- Both automated and manual checks must pass before merge

**Version**: 1.1.0 | **Ratified**: 2025-11-23 | **Last Amended**: 2025-11-25

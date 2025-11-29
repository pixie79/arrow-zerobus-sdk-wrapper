# Research: OTLP SDK Integration Update

**Feature**: 002-otlp-sdk-update  
**Date**: 2025-01-27

## Research Tasks

### 1. otlp-rust-service SDK API Analysis

**Task**: Understand the SDK API provided by otlp-rust-service for metrics and traces.

**Findings**:
- The otlp-rust-service provides `OtlpLibrary` as the main SDK entry point
- SDK is accessed via `otlp-arrow-library` crate (git dependency from https://github.com/pixie79/otlp-rust-service, main branch)
- Current usage shows:
  - `OtlpLibrary::new(config)` - async initialization
  - `library.export_metrics(metrics)` - export metrics
  - `library.export_trace(span_data)` - export traces
  - `library.flush()` - flush pending data
  - `library.shutdown()` - shutdown SDK
- Configuration uses `ConfigBuilder` pattern with:
  - `output_dir(path)` - output directory for file-based export
  - `write_interval_secs(seconds)` - write interval configuration

**Decision**: Use the SDK's native methods for metrics and trace collection. The SDK should provide proper methods for creating metrics and traces that replace manual construction.

**Rationale**: The SDK is designed to handle OpenTelemetry data construction and export internally, reducing the need for manual ResourceMetrics and SpanData construction. This aligns with the goal of using the SDK instead of manual construction.

**Alternatives Considered**:
- Continue using manual construction with SDK export - Rejected: Defeats the purpose of using the SDK
- Direct OpenTelemetry SDK usage - Rejected: We want to use otlp-rust-service SDK for consistency

### 2. SDK Methods for Metrics and Traces

**Task**: Identify SDK methods that replace manual construction of metrics and traces.

**Findings**:
- Current implementation manually constructs:
  - `ResourceMetrics` via `create_batch_metrics()` - currently returns empty/default structure
  - `SpanData` via `create_span_data()` - manually constructs with random IDs, attributes, etc.
- The SDK should provide methods to:
  - Record metrics directly (replacing manual ResourceMetrics construction)
  - Create spans using SDK's span builder (replacing manual SpanData construction)
- SDK likely provides higher-level APIs that handle:
  - Metric recording with proper OpenTelemetry structure
  - Span creation with proper trace context management
  - Automatic export handling

**Decision**: Replace `create_batch_metrics()` and `create_span_data()` with SDK-provided methods. The exact API will be determined during implementation by examining the SDK documentation or source code.

**Rationale**: Using SDK methods ensures proper OpenTelemetry structure, trace context propagation, and reduces code maintenance burden. The SDK handles the complexity of OpenTelemetry data structures internally.

**Alternatives Considered**:
- Keep manual construction but improve it - Rejected: Spec requires using SDK methods
- Hybrid approach - Rejected: Spec requires full SDK usage

### 3. Configuration Structure Alignment

**Task**: Determine how to align OtlpConfig with SDK requirements.

**Findings**:
- Current `OtlpConfig` has:
  - `endpoint: Option<String>` - OTLP endpoint URL
  - `log_level: String` - Log level for tracing
  - `extra: HashMap<String, Value>` - Additional options
- Current `convert_config()` method:
  - Maps endpoint to output_dir (incorrect mapping)
  - Sets write_interval_secs to fixed 5 seconds
  - Sets RUST_LOG environment variable
- SDK `Config` likely needs:
  - Proper endpoint configuration (not just output_dir)
  - Write interval configuration
  - Log level configuration (may be handled differently)

**Decision**: Simplify `OtlpConfig` to directly map to SDK `Config` requirements. Remove the `convert_config()` method and use SDK's `ConfigBuilder` directly or create a simpler mapping. Allow breaking changes to configuration structure.

**Rationale**: Direct alignment with SDK configuration reduces conversion complexity and potential bugs. Breaking changes are acceptable since no users currently depend on the wrapper.

**Alternatives Considered**:
- Keep conversion layer - Rejected: Adds unnecessary complexity
- Maintain backward compatibility - Rejected: Spec allows breaking changes

### 4. Dead Code Removal Strategy

**Task**: Identify all code to be removed as part of the migration.

**Findings**:
- Methods to remove:
  - `new()` - synchronous method that always returns None (dead code)
  - `create_batch_metrics()` - manual metrics construction (lines 159-194)
  - `create_span_data()` - manual span construction (lines 295-344)
  - `convert_config()` - configuration conversion (lines 86-120)
- Code patterns to remove:
  - Manual ResourceMetrics construction
  - Manual SpanData construction with random IDs
  - Tracing log-based metric export workaround
  - Configuration conversion logic

**Decision**: Remove all identified dead code. Replace with SDK method calls. Update tests to use new SDK-based API.

**Rationale**: Removing dead code simplifies the codebase and reduces maintenance burden. The SDK provides all necessary functionality.

**Alternatives Considered**:
- Keep methods as deprecated - Rejected: No users, clean removal preferred
- Gradual migration - Rejected: Spec requires complete migration

### 5. Error Handling and Initialization

**Task**: Understand how SDK handles initialization failures and errors.

**Findings**:
- Current implementation:
  - Returns `None` if initialization fails
  - Logs warnings but continues without observability
  - Export failures are logged but don't interrupt operations
- SDK likely provides:
  - Result types for initialization
  - Error types for export operations
  - Graceful degradation when SDK unavailable

**Decision**: Maintain current error handling pattern: graceful degradation when SDK fails, logging warnings, continuing without observability. Use SDK's error types where appropriate.

**Rationale**: Maintains existing behavior while using SDK. Ensures data transmission operations are not interrupted by observability failures.

**Alternatives Considered**:
- Fail fast on SDK errors - Rejected: Would break data transmission
- Retry SDK operations - Rejected: Observability is non-critical

## Summary

The migration involves:
1. Using SDK's native methods for metrics and trace creation (replacing manual construction)
2. Simplifying configuration to align with SDK requirements (removing conversion layer)
3. Removing all dead code (manual construction methods, unused initialization method)
4. Maintaining graceful error handling (continue without observability on failures)

The exact SDK API methods will be determined during implementation by examining the otlp-rust-service SDK source code or documentation.


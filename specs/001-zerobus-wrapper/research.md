# Research: Zerobus SDK Wrapper

**Feature**: 001-zerobus-wrapper  
**Date**: 2025-11-23  
**Status**: Complete

## Research Tasks

### 1. Databricks Zerobus SDK Integration Patterns

**Task**: Research best practices for integrating with databricks-zerobus-ingest-sdk, including initialization, authentication, and data transmission patterns.

**Findings**:
- The Zerobus SDK uses gRPC streaming for data ingestion
- Requires OAuth2 credentials (client_id, client_secret) and Unity Catalog URL
- Endpoint format: `https://{workspace-id}.cloud.databricks.com` or `https://{workspace-id}.zerobus.{region}.cloud.databricks.com`
- SDK expects Protobuf FileDescriptorProto for schema definition
- Stream creation is lazy (created on first write)
- SDK handles gRPC connection management internally

**Decision**: Reuse ZerobusSDKWriter pattern from cap-gl-consumer-rust/src/writer/zerobus_sdk.rs as the foundation, adapting for wrapper API design.

**Rationale**: The reference implementation in cap-gl-consumer-rust provides proven patterns for SDK integration, authentication handling, and error management that can be adapted for the wrapper.

**Alternatives Considered**:
- Direct SDK usage without wrapper layer - Rejected: Need unified API and additional features (retry, observability, debug output)
- Custom gRPC implementation - Rejected: SDK provides tested, maintained implementation

### 2. Arrow to Protobuf Conversion Strategy

**Task**: Research efficient Arrow RecordBatch to Protobuf conversion patterns, ensuring compatibility with Zerobus requirements.

**Findings**:
- Zerobus requires Protobuf format with FileDescriptorProto for schema
- Arrow RecordBatch contains schema and data that must be converted
- Conversion requires mapping Arrow types to Protobuf types
- Schema inference may be needed if not provided
- Protobuf descriptor generation is required for SDK stream creation

**Decision**: Reuse arrow_to_protobuf conversion logic from cap-gl-consumer-rust/src/databricks_sdk/arrow_to_protobuf.rs and src/writer/arrow_to_protobuf.rs, adapting for wrapper use case.

**Rationale**: The reference implementation has proven conversion logic that handles schema mapping, type conversion, and descriptor generation. Reusing this ensures compatibility and reduces risk.

**Alternatives Considered**:
- Custom conversion implementation - Rejected: Higher risk, more development time
- Using Arrow Flight directly - Rejected: Zerobus requires Protobuf format

### 3. Retry Strategy with Exponential Backoff and Jitter

**Task**: Research implementation patterns for exponential backoff with jitter for retry logic in Rust async contexts.

**Findings**:
- Exponential backoff: delay = base_delay * (2 ^ attempt_number)
- Jitter prevents thundering herd problem by adding randomness
- Common jitter strategies: full jitter, equal jitter, decorrelated jitter
- Tokio provides `tokio::time::sleep` for async delays
- Retry crates available: `tokio-retry`, but custom implementation may be preferred for control

**Decision**: Implement custom retry logic using exponential backoff with full jitter (random delay between 0 and calculated exponential delay). Use tokio::time::sleep for async delays.

**Rationale**: Custom implementation provides full control over retry behavior, aligns with constitution requirements for clarity, and allows fine-tuning for Zerobus-specific error patterns.

**Alternatives Considered**:
- Using tokio-retry crate - Rejected: Less control, additional dependency
- Fixed delay retry - Rejected: Less efficient, doesn't handle transient failures well

### 4. OpenTelemetry Integration via otlp-rust-service

**Task**: Research integration patterns for using otlp-rust-service library for OpenTelemetry metrics and traces.

**Findings**:
- otlp-rust-service provides OtlpLibrary API for embedded usage
- Supports both metrics and traces export
- Can be configured for file output or remote forwarding
- Uses Arrow IPC format for efficient data transfer
- Provides Python bindings that can be reused

**Decision**: Integrate otlp-arrow-library (from https://github.com/pixie79/otlp-rust-service, main branch) as a git dependency, using its OtlpLibrary API for metrics and trace collection. Configure it to export to standard OpenTelemetry backends. The main branch includes fixes for ResourceMetrics issues.

**Rationale**: Reusing otlp-rust-service provides proven OpenTelemetry integration, reduces code duplication, and ensures consistency with other projects using the same library. Using the main branch ensures we have the latest fixes, including ResourceMetrics handling improvements.

**Alternatives Considered**:
- Direct OpenTelemetry SDK usage - Rejected: More code, less reuse
- Custom observability implementation - Rejected: Reinventing wheel, higher maintenance

### 5. PyO3 Bindings Best Practices

**Task**: Research best practices for creating Python bindings with PyO3, ensuring zero-copy data transfer and Pythonic API design.

**Findings**:
- PyO3 0.20 supports async/await patterns via tokio runtime
- Zero-copy transfer possible with Arrow data (PyArrow integration)
- Python naming conventions: snake_case for functions, classes follow Python patterns
- Error handling: Convert Rust errors to Python exceptions
- Memory management: PyO3 handles Rust/Python boundary automatically
- Async functions require tokio runtime in Python bindings

**Decision**: Use PyO3 0.20 with auto-initialize and extension-module features. Create Python classes that wrap Rust structs, use snake_case naming, and provide async methods where needed. Leverage PyArrow for zero-copy Arrow data transfer.

**Rationale**: PyO3 is the standard for Rust-Python interop, provides excellent performance, and handles memory management correctly. Following Python conventions ensures good developer experience.

**Alternatives Considered**:
- ctypes/CFFI bindings - Rejected: More complex, less type-safe
- Separate Python implementation - Rejected: Violates constitution requirement for shared implementation

### 6. Thread-Safety and Concurrent Access Patterns

**Task**: Research Rust patterns for thread-safe concurrent access to shared state in async contexts.

**Findings**:
- Arc<Mutex<T>> for shared mutable state in async contexts
- Arc<RwLock<T>> for read-heavy workloads
- Tokio's Mutex for async-aware locking (prevents blocking)
- Send + Sync traits required for cross-thread usage
- Internal synchronization needed for concurrent operations

**Decision**: Use Arc<tokio::sync::Mutex<T>> for internal state management. Ensure all public types are Send + Sync. Use async-aware locks to prevent blocking the async runtime.

**Rationale**: Tokio Mutex is designed for async contexts and prevents blocking. Arc enables shared ownership across threads. This pattern is standard in Rust async applications.

**Alternatives Considered**:
- Synchronous Mutex - Rejected: Can block async runtime
- Lock-free data structures - Rejected: More complex, may not be necessary

### 7. Authentication Token Refresh Strategy

**Task**: Research patterns for automatic OAuth2 token refresh in long-running applications.

**Findings**:
- OAuth2 tokens have expiration times
- Refresh tokens can be used to obtain new access tokens
- Token refresh should be transparent to caller
- Need to detect 401/403 errors indicating expired tokens
- Refresh should happen before retry logic

**Decision**: Implement token refresh detection by monitoring authentication errors from Zerobus SDK. When token expiration is detected, refresh the token using OAuth2 refresh flow, then retry the operation transparently.

**Rationale**: Transparent token refresh provides better user experience and aligns with production SDK patterns. Detecting expiration via error codes is reliable and doesn't require token introspection.

**Alternatives Considered**:
- Proactive token refresh (before expiration) - Rejected: More complex, requires token introspection
- Caller-managed refresh - Rejected: Violates requirement for transparent operation

### 8. Debug File Output and Rotation

**Task**: Research efficient file writing patterns for Arrow IPC and Protobuf files with rotation and flushing.

**Findings**:
- Arrow IPC streaming format supports append operations
- File rotation needed to prevent unbounded growth
- Periodic flushing ensures data is written even if process crashes
- Timestamp-based or size-based rotation strategies
- Arrow IPC files can be read by standard Arrow libraries

**Decision**: Implement size-based rotation with configurable max file size. Use periodic flush interval (default 5 seconds) to ensure data is written. Write Arrow IPC format for RecordBatches and binary Protobuf format for converted data.

**Rationale**: Size-based rotation prevents unbounded disk usage. Periodic flushing ensures data durability. Standard formats (Arrow IPC, Protobuf) enable easy inspection with standard tools.

**Alternatives Considered**:
- Time-based rotation only - Rejected: Doesn't handle large batches well
- No rotation - Rejected: Risk of unbounded disk usage

## Summary

All research tasks completed. Key decisions:
1. Reuse ZerobusSDKWriter patterns from cap-gl-consumer-rust
2. Reuse Arrow to Protobuf conversion logic from reference implementation
3. Custom retry with exponential backoff + full jitter
4. Integrate otlp-rust-service for OpenTelemetry
5. PyO3 bindings with Pythonic API design
6. Arc<tokio::sync::Mutex> for thread-safety
7. Transparent token refresh on authentication errors
8. Size-based file rotation with periodic flushing

All decisions align with constitution requirements and enable code reuse from cap-gl-consumer-rust for drop-in replacement compatibility.


# Feature Specification: OTLP SDK Integration Update

**Feature Branch**: `002-otlp-sdk-update`  
**Created**: 2025-01-27  
**Status**: Draft  
**Input**: User description: "update dependancy on otlp-rust-service, we should use the sdk from the service for metrics and logging via traces."

## Clarifications

### Session 2025-01-27

- Q: What is the scope of API and configuration changes allowed? Can we modify OtlpConfig structure and ObservabilityManager public API? → A: Allow breaking changes to both API and configuration (can modify OtlpConfig and ObservabilityManager)
- Q: What specific dead code should be removed? → A: Remove all manual observability construction methods (create_batch_metrics, create_span_data, convert_config) and unused code paths, including the synchronous `new()` method that always returns None

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Use OTLP Service SDK for Metrics and Traces (Priority: P1)

A developer or operations team needs the wrapper to use the standardized SDK from otlp-rust-service for metrics collection and logging via traces, rather than manually constructing observability data. This ensures consistency with the service's SDK patterns and reduces maintenance burden by leveraging the service's built-in capabilities.

**Why this priority**: This is a foundational change that affects how observability is implemented. Using the service SDK ensures proper integration and reduces custom code that needs to be maintained.

**Independent Test**: Can be fully tested by initializing the wrapper with observability enabled, performing data transmission operations, and verifying that metrics and traces are generated using the otlp-rust-service SDK rather than manual construction. The test delivers value by confirming the wrapper uses the standardized SDK approach.

**Acceptance Scenarios**:

1. **Given** a wrapper instance with observability enabled, **When** data is sent to Zerobus, **Then** metrics are recorded using the otlp-rust-service SDK methods
2. **Given** a wrapper instance with observability enabled, **When** data transmission operations occur, **Then** traces are generated using the otlp-rust-service SDK for logging via traces
3. **Given** a wrapper instance with observability configured, **When** operations complete, **Then** observability data is exported using the SDK's built-in export mechanisms
4. **Given** a wrapper instance, **When** observability is initialized, **Then** the otlp-rust-service SDK is used for all metrics and trace operations instead of manual construction

---

### Edge Cases

- What happens when the otlp-rust-service SDK is unavailable or fails to initialize? The wrapper should handle initialization failures gracefully, logging warnings and continuing without observability
- How does the wrapper handle SDK version compatibility issues? The wrapper should use a compatible version of the SDK and handle version mismatches with clear error messages
- What occurs when SDK export operations fail? The wrapper should log errors but not interrupt data transmission operations
- How does the system handle SDK configuration changes? The wrapper should support configuration updates through the SDK-aligned configuration structure
- What happens when metrics or trace data cannot be exported via the SDK? The wrapper should log the failure but continue normal operations

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST update the dependency on otlp-rust-service to use the SDK from the service
- **FR-002**: System MUST use the otlp-rust-service SDK for metrics collection instead of manual metrics construction
- **FR-003**: System MUST use the otlp-rust-service SDK for logging via traces instead of manual trace construction
- **FR-004**: System MUST preserve existing observability functionality (metrics recording, trace generation, export) while using the SDK
- **FR-005**: System MUST handle SDK initialization failures gracefully without breaking data transmission operations
- **FR-006**: System MUST use SDK-provided methods for exporting metrics and traces
- **FR-007**: System MUST remove manual construction of observability data structures where SDK provides equivalent functionality
- **FR-008**: System MUST remove all dead code including: create_batch_metrics method, create_span_data method, convert_config method, and the synchronous `new()` method that always returns None
- **FR-009**: System MUST ensure metrics and traces are properly formatted according to SDK standards
- **FR-010**: System MUST simplify configuration structure to align with SDK requirements (breaking changes allowed)

### Key Entities *(include if feature involves data)*

- **OTLP SDK Configuration**: Represents the configuration for the otlp-rust-service SDK, including endpoint settings, export configuration, and logging levels, structured to align with SDK requirements

- **Observability Operations**: Represents metrics recording and trace generation operations that now use the SDK instead of manual construction, maintaining the same functional behavior but with SDK-provided implementations

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of metrics recording operations use the otlp-rust-service SDK instead of manual metrics construction
- **SC-002**: 100% of trace generation operations use the otlp-rust-service SDK for logging via traces instead of manual trace construction
- **SC-003**: The wrapper maintains feature parity with existing observability functionality, with all previously supported metrics and traces still available
- **SC-004**: SDK initialization succeeds in 99.9% of cases when valid configuration is provided
- **SC-005**: When observability is enabled, metrics and traces are exported successfully using SDK methods with the same reliability as the previous implementation
- **SC-006**: All dead code is removed including: create_batch_metrics, create_span_data, convert_config methods, and the synchronous `new()` method that always returns None
- **SC-007**: All existing observability tests pass with the SDK-based implementation (tests may be updated to reflect new API)
- **SC-008**: The wrapper's observability behavior is consistent with otlp-rust-service SDK patterns and conventions

## Assumptions

- The otlp-rust-service SDK provides methods for metrics collection and trace generation that can replace the current manual construction approach
- The SDK maintains compatibility with OpenTelemetry standards for metrics and traces
- The SDK handles export operations internally, reducing the need for manual export code
- Updating to use the SDK will simplify the codebase by removing manual observability data construction and dead code
- Configuration structure can be modified to align with SDK requirements without maintaining backward compatibility

## Dependencies

- otlp-rust-service repository must provide an SDK with metrics and trace capabilities
- The SDK must be compatible with the current OpenTelemetry version (0.31) used by the wrapper
- The SDK must support async initialization and operations to match the wrapper's async architecture

## Out of Scope

- Migration of existing observability data formats (assumes SDK compatibility)
- Performance optimizations beyond what the SDK provides (uses SDK defaults)
- Maintaining compatibility with any existing wrapper consumers (no current users)

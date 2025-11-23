# Feature Specification: Zerobus SDK Wrapper

**Feature Branch**: `001-zerobus-wrapper`  
**Created**: 2025-11-23  
**Status**: Draft  
**Input**: User description: "This is designed to be a wrapper for the databricks zerobus-rust-sdk. It should be useable cross platform and also the public methods should be callable from both rust and python applications. We should include opentelmetry metrics and logging via opentelemetry traces. To do this we should make use of the lib here /Users/mark.olliver/GIT/otlp-rust-service. A sample of the working code for this project can be found /Users/mark.olliver/GIT/cap-gl-consumer-rust and we should reuse as much as possible so that we can swop this lib into that project in the future. We should enable config to be passed that allows us to write out debug files for both ARROW RecordBatchs and also the converted Proto files that would be sent to Zerobus. This should be written to {OUTPUT_DIR}/zerobus/arrow/table.arrow and {OUTPUT_DIR}/zerobus/proto/table.proto. We should allow configuration for how often arrow IPC files are written, The max size and rotation. We should also have switches to enable/disable the debug files with the default being disabled."

## Clarifications

### Session 2025-11-23

- Q: How should the wrapper handle transient failures (network issues, temporary service unavailability)? Should it retry automatically, and if so, with what strategy? → A: Automatic retry with exponential backoff with jitter, configurable max retries (default: 5), then return error
- Q: What should happen when a data batch exceeds size limits? Should the wrapper reject it, split it, or handle it differently? → A: Larger batches should be handled and passed to Zerobus SDK (no rejection). The 10MB limit mentioned in success criteria is specifically for ensuring debug files are continuously written. Debug files must be flushed to disk at least every 5 seconds (configurable) to ensure continuous writing even with larger batches
- Q: Should the wrapper be thread-safe and support concurrent operations from multiple threads/async tasks, or require serialized access? → A: Thread-safe: Multiple threads/async tasks can call the wrapper concurrently, with internal synchronization
- Q: When authentication tokens expire during long-running operations, should the wrapper automatically refresh them, or require the caller to re-authenticate? → A: Automatic token refresh: Wrapper detects expiration, refreshes token transparently, and retries the operation
- Q: What is the minimum Python version requirement? → A: Python 3.11+ (minimum supported version)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Send Data to Zerobus from Rust Application (Priority: P1)

A Rust application developer needs to send structured data to Databricks Zerobus using a simple, consistent API. They want to initialize the wrapper with configuration, send data batches, and have the wrapper handle all the complexity of protocol conversion, authentication, and network communication. The wrapper should provide clear error messages if something goes wrong.

**Why this priority**: This is the core functionality - without the ability to send data from Rust, the wrapper provides no value. This must work reliably as it's the primary use case.

**Independent Test**: Can be fully tested by creating a Rust application that initializes the wrapper, sends a batch of test data, and verifies successful delivery to Zerobus. The test delivers value by confirming the wrapper can successfully transmit data.

**Acceptance Scenarios**:

1. **Given** a Rust application with valid Zerobus credentials, **When** the application initializes the wrapper and sends a data batch, **Then** the data is successfully transmitted to Zerobus and the application receives confirmation
2. **Given** a Rust application with invalid credentials, **When** the application attempts to initialize the wrapper, **Then** the application receives a clear error message explaining the authentication failure
3. **Given** a Rust application that has sent data successfully, **When** the application sends subsequent batches, **Then** all batches are transmitted successfully without requiring re-initialization

---

### User Story 2 - Send Data to Zerobus from Python Application (Priority: P1)

A Python application developer needs to send structured data to Databricks Zerobus by calling the Rust SDK through Python bindings. They want the Python interface to feel natural and follow Python conventions while providing the same capabilities as the Rust interface, with the underlying implementation being the shared Rust SDK.

**Why this priority**: Multi-language support is a core requirement. Python developers must be able to use the wrapper with the same reliability and ease as Rust developers. This is equally critical as the Rust interface.

**Independent Test**: Can be fully tested by creating a Python application that imports the wrapper, initializes it with configuration, sends a batch of test data, and verifies successful delivery. The test delivers value by confirming Python developers can use the wrapper effectively.

**Acceptance Scenarios**:

1. **Given** a Python application with valid Zerobus credentials, **When** the application initializes the wrapper and sends a data batch, **Then** the data is successfully transmitted to Zerobus and the application receives confirmation
2. **Given** a Python application, **When** the application uses the wrapper API, **Then** the API follows Python naming conventions and error handling patterns
3. **Given** a Python application that has sent data successfully, **When** the application sends subsequent batches, **Then** all batches are transmitted successfully with consistent behavior to the Rust interface

---

### User Story 3 - Monitor Operations with OpenTelemetry Observability (Priority: P2)

A developer or operations team needs visibility into wrapper operations through standardized observability metrics and traces. They want to understand performance characteristics, identify bottlenecks, and troubleshoot issues without needing to instrument the wrapper code themselves.

**Why this priority**: Observability is critical for production use but not required for basic functionality. It enables monitoring, debugging, and performance optimization, making it important but secondary to core data transmission.

**Independent Test**: Can be fully tested by initializing the wrapper with observability enabled, performing data transmission operations, and verifying that metrics and traces are generated and can be exported to standard observability backends. The test delivers value by confirming operational visibility is available.

**Acceptance Scenarios**:

1. **Given** a wrapper instance with observability enabled, **When** data is sent to Zerobus, **Then** metrics are recorded for operation counts, latencies, and success/failure rates
2. **Given** a wrapper instance with observability enabled, **When** data transmission operations occur, **Then** traces are generated showing the flow of operations and timing information
3. **Given** a wrapper instance with observability configured, **When** operations complete, **Then** observability data can be exported to standard OpenTelemetry-compatible backends

---

### User Story 4 - Debug Data Flow with File Output (Priority: P3)

A developer needs to debug data transformation issues by inspecting the exact data being processed. They want to see both the Arrow RecordBatch format and the converted Protobuf format that would be sent to Zerobus, written to local files for inspection. This should be configurable and disabled by default to avoid performance impact in production.

**Why this priority**: Debug capabilities are valuable for development and troubleshooting but not required for normal operation. This is a developer convenience feature that should not impact production performance.

**Independent Test**: Can be fully tested by enabling debug file output, performing data transmission operations, and verifying that Arrow and Protobuf files are written to the configured output directory with the expected content. The test delivers value by confirming developers can inspect data transformations.

**Acceptance Scenarios**:

1. **Given** a wrapper instance with debug file output enabled, **When** data is sent to Zerobus, **Then** Arrow RecordBatch files are written to the configured output directory
2. **Given** a wrapper instance with debug file output enabled, **When** data is converted for transmission, **Then** Protobuf files are written showing the exact format that would be sent
3. **Given** a wrapper instance with debug file output disabled (default), **When** data is sent to Zerobus, **Then** no debug files are created and performance is not impacted
4. **Given** a wrapper instance with debug file output enabled and rotation configured, **When** files reach the maximum size, **Then** new files are created and old files are rotated according to configuration
5. **Given** a wrapper instance with debug file output enabled, **When** data is being written to debug files, **Then** files are flushed to disk at least every 5 seconds (configurable) to ensure continuous writing

---

### Edge Cases

- What happens when network connectivity is lost during data transmission? The wrapper automatically retries with exponential backoff with jitter (configurable max retries, default: 5), then returns error to caller
- How does the wrapper handle Zerobus service unavailability or rate limiting? The wrapper automatically retries with exponential backoff with jitter (configurable max retries, default: 5), then returns error to caller
- What occurs when data batches exceed maximum size limits? Larger batches are handled and passed to Zerobus SDK without rejection. The 10MB reference in success criteria is for debug file continuity, not a hard limit for data transmission
- How does the system handle concurrent requests from multiple threads or async tasks? The wrapper is thread-safe and supports concurrent operations from multiple threads/async tasks with internal synchronization
- What happens when debug file output is enabled but the output directory is not writable?
- How does the wrapper behave when configuration values are invalid or missing required fields?
- What occurs when Arrow RecordBatch data cannot be converted to Protobuf format?
- How does the system handle partial failures in batch operations?
- What happens when observability export fails but data transmission succeeds?
- How does the wrapper handle authentication token expiration during long-running operations? The wrapper automatically detects token expiration, refreshes the token transparently, and retries the operation without requiring caller intervention

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a Rust SDK API for initializing the wrapper with Zerobus connection configuration
- **FR-002**: System MUST provide Python bindings that allow calling the Rust SDK from Python applications (minimum Python version 3.11)
- **FR-003**: System MUST accept structured data in Arrow RecordBatch format from both Rust and Python applications
- **FR-004**: System MUST convert Arrow RecordBatch data to Protobuf format required by Zerobus
- **FR-005**: System MUST transmit converted data to Zerobus using the underlying Zerobus SDK
- **FR-006**: System MUST handle authentication with Zerobus using provided credentials
- **FR-007**: System MUST provide consistent error messages and error handling across Rust and Python interfaces
- **FR-008**: System MUST support cross-platform operation (Linux, macOS, Windows)
- **FR-009**: System MUST integrate OpenTelemetry metrics collection for operation monitoring
- **FR-010**: System MUST integrate OpenTelemetry trace collection for operation observability
- **FR-011**: System MUST use the otlp-rust-service library for OpenTelemetry functionality
- **FR-012**: System MUST provide configuration to enable or disable debug file output (default: disabled)
- **FR-013**: System MUST write Arrow RecordBatch debug files to {OUTPUT_DIR}/zerobus/arrow/table.arrow when enabled
- **FR-014**: System MUST write Protobuf debug files to {OUTPUT_DIR}/zerobus/proto/table.proto when enabled
- **FR-015**: System MUST support configuration for Arrow IPC file write frequency
- **FR-016**: System MUST support configuration for maximum file size before rotation
- **FR-017**: System MUST support file rotation when maximum size is reached
- **FR-023**: System MUST accept data batches of any size and pass them to Zerobus SDK without rejection
- **FR-024**: System MUST flush debug files to disk at least every 5 seconds (configurable flush interval) to ensure continuous writing even with larger batches
- **FR-018**: System MUST reuse code patterns and structures from the cap-gl-consumer-rust project where applicable
- **FR-019**: System MUST be designed to be a drop-in replacement for zerobus functionality in cap-gl-consumer-rust
- **FR-020**: System MUST provide clear, actionable error messages when operations fail
- **FR-021**: System MUST automatically retry transient failures (network issues, temporary service unavailability) using exponential backoff with jitter, with configurable maximum retry count (default: 5 retries)
- **FR-022**: System MUST return error to caller after exhausting retry attempts for transient failures
- **FR-025**: System MUST be thread-safe, allowing multiple threads and async tasks to call the wrapper concurrently with internal synchronization
- **FR-026**: System MUST automatically detect authentication token expiration and refresh tokens transparently, retrying operations without requiring caller intervention

### Key Entities *(include if feature involves data)*

- **Wrapper Configuration**: Represents the complete configuration for initializing the wrapper, including Zerobus connection details (endpoint, credentials), observability settings (enabled/disabled, export configuration), debug file settings (enabled/disabled, output directory, write frequency, max size, rotation policy, flush interval), and retry settings (max retries, backoff configuration)

- **Data Batch**: Represents a collection of structured data records in Arrow RecordBatch format that needs to be transmitted to Zerobus. Contains schema information and record data that will be converted to Protobuf format

- **Transmission Result**: Represents the outcome of a data transmission operation, including success/failure status, error information if applicable, and any relevant metadata about the transmission

- **Observability Data**: Represents collected metrics and traces about wrapper operations, including operation counts, latencies, success rates, and trace spans showing operation flow

- **Debug Files**: Represents the Arrow and Protobuf file outputs written to disk when debug mode is enabled, containing the exact data formats being processed and transmitted

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can successfully send data batches to Zerobus from both Rust and Python applications with 99.999% success rate under normal network conditions
- **SC-002**: Data transmission operations complete within acceptable latency thresholds (p95 latency under 150ms for batches up to 10MB). Note: The 10MB reference is for debug file continuity measurement; larger batches are accepted and transmitted
- **SC-003**: The wrapper API can be integrated into existing applications (like cap-gl-consumer-rust) with minimal code changes, requiring changes to fewer than 5% of existing code lines
- **SC-004**: When observability is enabled, 100% of data transmission operations generate corresponding metrics and traces
- **SC-005**: When debug file output is enabled, 100% of data batches result in corresponding Arrow and Protobuf files being written to the configured output directory
- **SC-006**: The wrapper maintains feature parity between Rust SDK and Python bindings, with identical functionality available through both interfaces (Python bindings call the Rust SDK)
- **SC-007**: Error messages provide sufficient information for developers to diagnose and resolve issues in 90% of failure scenarios without requiring additional debugging tools
- **SC-008**: The wrapper operates correctly on all target platforms (Linux, macOS, Windows) with identical behavior across platforms
- **SC-009**: When debug file output is disabled (default), file I/O operations have zero measurable performance impact on data transmission operations
- **SC-010**: File rotation occurs correctly when maximum size is reached, with no data loss and proper file naming to maintain chronological order
- **SC-011**: When debug file output is enabled, debug files are flushed to disk at least every 5 seconds (or configured interval) to ensure continuous writing regardless of batch size

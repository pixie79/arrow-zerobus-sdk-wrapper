# Feature Specification: Add Per-Row Error Information to TransmissionResult

**Feature Branch**: `004-per-row-errors`  
**Created**: 2025-01-27  
**Status**: Draft  
**Input**: User description: "Add Per-Row Error Information to TransmissionResult"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Partial Batch Success Handling (Priority: P1)

When sending a batch of data rows to Zerobus, some rows may fail due to serialization errors, type mismatches, or missing nested descriptors, while other rows are valid. Users need to identify which specific rows failed so they can quarantine only the failed rows and successfully write the valid rows to the main table.

**Why this priority**: This is the core value proposition - preventing unnecessary data loss by enabling partial success handling. Without this, all rows in a batch are quarantined even if only a subset failed, leading to significant data loss in production systems processing large batches (100-20,000 rows).

**Independent Test**: Can be fully tested by sending a batch with mixed valid and invalid rows, verifying that the TransmissionResult identifies successful and failed row indices, and confirming that only failed rows are quarantined while successful rows are written.

**Acceptance Scenarios**:

1. **Given** a batch with 100 rows where 5 rows have serialization errors, **When** sending the batch via `send_batch()`, **Then** TransmissionResult indicates 95 successful rows and 5 failed rows with specific error details for each failed row
2. **Given** a batch with mixed valid and invalid rows, **When** examining the TransmissionResult, **Then** users can identify exact row indices that failed along with specific error messages for each failure
3. **Given** a batch where all rows succeed, **When** sending the batch, **Then** TransmissionResult indicates all rows succeeded with no failed_rows entries
4. **Given** a batch where all rows fail, **When** sending the batch, **Then** TransmissionResult indicates all rows failed with per-row error details

---

### User Story 2 - Efficient Quarantine Processing (Priority: P2)

Users processing batches from message queues (Kafka/Pulsar) need to efficiently quarantine only failed rows without losing successful data, reducing unnecessary data loss and improving system reliability.

**Why this priority**: This enables efficient error handling workflows in production systems where batches are processed continuously. It reduces operational overhead by minimizing false positives in quarantine systems.

**Independent Test**: Can be fully tested by implementing a quarantine workflow that processes TransmissionResult, extracts failed row indices, and quarantines only those specific rows while writing successful rows to the main table.

**Acceptance Scenarios**:

1. **Given** a TransmissionResult with failed_rows information, **When** processing the result, **Then** users can extract specific row indices and errors to quarantine only failed rows
2. **Given** a batch with partial failures, **When** quarantining failed rows, **Then** successful rows are not included in quarantine and are written to the main table
3. **Given** per-row error information, **When** analyzing error patterns, **Then** users can identify common failure causes across multiple rows

---

### User Story 3 - Enhanced Observability and Debugging (Priority: P3)

Users need detailed error information per row to diagnose issues, understand failure patterns, and improve data quality over time.

**Why this priority**: While valuable for debugging and monitoring, this is secondary to the core functionality of preventing data loss. It enables better observability but doesn't block the primary use case.

**Independent Test**: Can be fully tested by verifying that per-row error information includes sufficient detail (error type, message, row index) to enable debugging and pattern analysis.

**Acceptance Scenarios**:

1. **Given** a batch with failures, **When** examining TransmissionResult, **Then** each failed row includes error type and descriptive message for debugging
2. **Given** multiple batches with similar failures, **When** analyzing per-row errors, **Then** users can identify patterns (e.g., specific fields causing issues, common error types)
3. **Given** per-row error information, **When** monitoring system health, **Then** users can track failure rates and error distributions across batches

---

### Edge Cases

- What happens when a batch contains zero rows?
- How does system handle network errors that affect the entire batch versus individual rows?
- What happens when some rows fail during serialization but others fail during transmission?
- How does system handle errors that occur before individual row processing begins (e.g., authentication failures, connection errors)?
- What happens when retry logic is exhausted - are per-row errors preserved across retries?
- How does system handle very large batches (20,000+ rows) with many failures - is there a performance impact?
- What happens when all rows in a batch fail with the same error - should it be reported as batch-level or per-row?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide per-row error information in TransmissionResult when some rows in a batch fail
- **FR-002**: System MUST identify which specific rows succeeded and which failed by row index
- **FR-003**: System MUST provide specific error details (error type and message) for each failed row
- **FR-004**: System MUST maintain backward compatibility - existing code that doesn't use per-row information continues to work
- **FR-005**: System MUST support partial success scenarios where some rows succeed and others fail
- **FR-006**: System MUST provide total row count, successful count, and failed count in TransmissionResult
- **FR-007**: System MUST handle batch-level errors (e.g., authentication failures, connection errors) separately from per-row errors
- **FR-008**: System MUST preserve per-row error information across retry attempts when applicable
- **FR-009**: System MUST handle serialization errors at the row level (e.g., missing descriptors, type mismatches)
- **FR-010**: System MUST handle transmission errors at the row level when individual rows fail during transmission
- **FR-011**: System MUST provide empty or None values for per-row error fields when all rows succeed
- **FR-012**: System MUST provide empty or None values for successful_rows when all rows fail

### Key Entities *(include if feature involves data)*

- **TransmissionResult**: Result structure returned from batch transmission operations. Contains batch-level success/failure status, error information, retry attempts, latency, and new per-row error tracking fields (failed_rows, successful_rows, total_rows, successful_count, failed_count).

- **Failed Row Entry**: Represents a single failed row with its index (0-based position in batch) and associated error (ZerobusError type with specific error message).

- **Successful Row Entry**: Represents a single successful row by its index (0-based position in batch).

- **ZerobusError**: Error type that can represent various failure scenarios (ConversionError, TransmissionError, ConnectionError, AuthenticationError, etc.) and can be associated with specific rows.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can identify which specific rows failed in a batch with 100% accuracy (no false positives or false negatives in row identification)
- **SC-002**: Users can successfully quarantine only failed rows while writing successful rows to the main table, reducing unnecessary data loss by at least 90% compared to batch-level error handling
- **SC-003**: System maintains backward compatibility - 100% of existing code using TransmissionResult continues to work without modification
- **SC-004**: Per-row error information is available for at least 95% of row-level failures (serialization errors, type mismatches, missing descriptors)
- **SC-005**: Users can process batches with partial failures and extract per-row error information without significant performance degradation (processing time increases by less than 10% compared to batch-level error handling)
- **SC-006**: Error information includes sufficient detail (error type and message) for users to diagnose and resolve issues in at least 90% of failure scenarios

## Assumptions

- Per-row error tracking is primarily needed for serialization/conversion errors and row-level transmission errors
- Batch-level errors (authentication, connection failures) will continue to be reported at the batch level
- The feature will be opt-in via optional fields to maintain backward compatibility
- Performance impact of tracking per-row errors is acceptable for typical batch sizes (100-20,000 rows)
- Users will primarily use this feature in consumer applications (like zerobus-consumer) that process batches from message queues
- Error information will be used for quarantine workflows and debugging purposes
- Row indices are 0-based and correspond to the order of rows in the input RecordBatch

## Dependencies

- Existing TransmissionResult structure
- Existing ZerobusError enum
- Existing batch transmission logic in ZerobusWrapper
- Existing conversion logic that processes rows individually

## Constraints

- Must maintain backward compatibility with existing TransmissionResult usage
- Must not significantly impact performance for successful batches
- Must handle edge cases gracefully (empty batches, all-success, all-failure scenarios)
- Must work with existing retry logic
- Must support both Rust and Python bindings

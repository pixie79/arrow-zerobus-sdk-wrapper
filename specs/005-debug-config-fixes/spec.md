# Feature Specification: Debug Output Configuration Fixes

**Feature Branch**: `005-debug-config-fixes`  
**Created**: 2025-12-12  
**Status**: Draft  
**Input**: User description: "take the current git changes and update the code to also include fixes for the following two github issues: https://github.com/pixie79/arrow-zerobus-sdk-wrapper/issues"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Independent Control of Arrow and Protobuf Debug Output (Priority: P1)

Users need the ability to enable Arrow debug file output independently from Protobuf debug file output. Currently, a single `debug_enabled` flag controls both, which prevents users from debugging only one format at a time. This is particularly useful when:
- Users want to inspect Arrow data without generating Protobuf files (reducing disk usage)
- Users want to inspect Protobuf serialization without Arrow files
- Users need to troubleshoot format-specific issues independently

**Why this priority**: This addresses a core usability issue where users cannot selectively enable debug output formats, leading to unnecessary disk usage and reduced flexibility in debugging workflows.

**Independent Test**: Can be fully tested by configuring the wrapper with only Arrow debug enabled (Protobuf disabled) and verifying that Arrow files are created while Protobuf files are not, and vice versa.

**Acceptance Scenarios**:

1. **Given** a wrapper configured with Arrow debug enabled and Protobuf debug disabled, **When** sending a batch, **Then** Arrow debug files are created in the output directory and Protobuf debug files are not created
2. **Given** a wrapper configured with Protobuf debug enabled and Arrow debug disabled, **When** sending a batch, **Then** Protobuf debug files are created in the output directory and Arrow debug files are not created
3. **Given** a wrapper configured with both Arrow and Protobuf debug enabled, **When** sending a batch, **Then** both Arrow and Protobuf debug files are created
4. **Given** a wrapper configured with both Arrow and Protobuf debug disabled, **When** sending a batch, **Then** no debug files are created

---

### User Story 2 - Prevent File Name Length Errors from Recursive Timestamp Appending (Priority: P1)

Users experience "File name too long" errors when debug files are rotated multiple times. The current implementation appends timestamps to rotated files, but if a file already contains a timestamp from a previous rotation, the timestamp is appended again, causing recursive timestamp accumulation (e.g., `table_20241212_120000_20241212_120100_20241212_120200.proto`). This eventually exceeds filesystem path length limits.

**Why this priority**: This is a critical bug that causes system failures when files are rotated multiple times, preventing users from using debug output in long-running processes.

**Independent Test**: Can be fully tested by simulating multiple file rotations and verifying that rotated file names contain only one timestamp suffix, not recursively appended timestamps.

**Acceptance Scenarios**:

1. **Given** a debug file that has been rotated once (contains one timestamp), **When** the file is rotated again, **Then** the new rotated file name contains only one timestamp suffix (not two)
2. **Given** a debug file that has been rotated multiple times, **When** inspecting the file names, **Then** each rotated file contains exactly one timestamp suffix
3. **Given** a debug file with a long table name, **When** the file is rotated multiple times, **Then** the file names never exceed filesystem path length limits (typically 255 characters for filename)
4. **Given** both Arrow and Protobuf debug files, **When** both are rotated multiple times, **Then** neither file type experiences recursive timestamp appending

---

### User Story 3 - Automatic File Retention Management (Priority: P1)

Users need automatic cleanup of old debug files to prevent unlimited disk space consumption. When debug files are rotated frequently, old files accumulate and consume disk space. The system should automatically maintain a configurable number of recent files (default: 10) per file type, deleting the oldest files when the limit is exceeded.

**Why this priority**: Prevents disk space exhaustion in long-running processes with frequent file rotations, ensuring predictable disk usage and reducing manual cleanup overhead.

**Independent Test**: Can be fully tested by simulating 11+ file rotations and verifying that only the last 10 rotated files remain, with oldest files automatically deleted.

**Acceptance Scenarios**:

1. **Given** a wrapper with retention limit of 10 files, **When** the 11th file is rotated, **Then** the oldest rotated file is automatically deleted, leaving exactly 10 files
2. **Given** a wrapper with retention limit configured to 0, **When** files are rotated, **Then** no files are deleted (unlimited retention)
3. **Given** both Arrow and Protobuf debug enabled with retention limit of 10, **When** files are rotated, **Then** Arrow and Protobuf files are managed independently (each maintains up to 10 files)
4. **Given** a wrapper with retention limit configured, **When** the currently active file is being written, **Then** the active file is not counted toward retention limit (only rotated files count)

---

### Edge Cases

- What happens when a user enables Arrow debug but not Protobuf debug, and the wrapper needs to write Protobuf descriptor files? (Descriptor files should still be written if either format is enabled)
- What happens when a rotated file name would exceed filesystem limits even with a single timestamp? (Use shorter naming scheme - e.g., sequential numbers instead of timestamps when base name is too long)
- How does the system handle file rotation when the base file name already contains underscores or timestamp-like patterns? (Detect timestamp pattern at end of filename only, before extension, to preserve original base name without timestamps)
- What happens if both Arrow and Protobuf debug are disabled but `debug_output_dir` is set? (Valid configuration - no files written, no validation error)
- How does the system handle configuration migration from the old single `debug_enabled` flag? (Should maintain backward compatibility)
- What happens when file deletion fails during retention cleanup? (Log error and continue rotation - deletion failure should not block file rotation)
- How are files ordered for deletion (oldest first)? (By filename timestamp or sequential number - parse from filename to determine order)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide separate configuration flags to enable/disable Arrow debug output independently from Protobuf debug output
- **FR-002**: System MUST allow users to enable Arrow debug output without enabling Protobuf debug output
- **FR-003**: System MUST allow users to enable Protobuf debug output without enabling Arrow debug output
- **FR-004**: System MUST prevent recursive timestamp appending in rotated debug file names
- **FR-005**: System MUST ensure rotated file names contain exactly one timestamp suffix, regardless of how many times the file has been rotated
- **FR-006**: System MUST extract the original base file name (without timestamps) when generating rotated file paths by detecting timestamp pattern at the end of filename only (before extension)
- **FR-011**: System MUST use a shorter naming scheme (e.g., sequential numbers) instead of timestamps when the base filename is too long to fit within filesystem limits with timestamp format
- **FR-007**: System MUST maintain backward compatibility with existing `debug_enabled` configuration (if present, should enable both formats by default)
- **FR-008**: System MUST support configuration via environment variables, YAML files, and programmatic API for both Arrow and Protobuf debug flags
- **FR-009**: System MUST write Protobuf descriptor files if either Arrow or Protobuf debug output is enabled
- **FR-010**: System MUST allow `debug_output_dir` to be set even when both Arrow and Protobuf debug flags are disabled (no validation error; no files will be written in this case)
- **FR-012**: System MUST maintain a configurable retention limit for rotated debug files (default: 10 files per type, configurable via `debug_max_files_retained`, allow 0 to disable retention)
- **FR-013**: System MUST delete the oldest rotated file immediately after rotation when retention limit is exceeded (when 11th file is created, delete the 1st)
- **FR-014**: System MUST apply retention limits independently per file type (Arrow files and Protobuf files have separate retention limits)
- **FR-015**: System MUST exclude the currently active file being written to from retention count (only rotated/closed files count toward limit)
- **FR-016**: System MUST determine file deletion order by parsing timestamp from filename (or sequential number if using sequential naming pattern) to identify oldest files

### Key Entities

- **Debug Configuration**: Represents the debug output settings, including separate flags for Arrow and Protobuf output, output directory, flush interval, max file size, and file retention limit
- **File Rotation State**: Tracks the current file path and rotation count to prevent recursive timestamp appending
- **Base File Name**: The original file name without any timestamp suffixes, used as the foundation for generating rotated file names

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can enable Arrow debug output independently from Protobuf debug output with 100% success rate
- **SC-002**: Rotated debug file names never exceed filesystem path length limits (255 characters for filename) even after 100+ rotations
- **SC-003**: Rotated debug file names contain exactly one timestamp suffix regardless of rotation count (verified across 100+ rotations)
- **SC-004**: Zero "File name too long" errors occur during debug file rotation in production workloads
- **SC-005**: Configuration changes for separate Arrow/Protobuf flags take effect immediately without requiring wrapper restart
- **SC-006**: Backward compatibility is maintained - existing configurations using single `debug_enabled` flag continue to work without modification
- **SC-007**: System maintains exactly the configured number of rotated debug files per type (default 10), automatically deleting oldest files when limit exceeded
- **SC-008**: Disk space usage for debug files remains bounded and predictable based on retention limit configuration

## Assumptions

- Users want granular control over debug output formats
- File rotation occurs frequently enough that recursive timestamp appending becomes a problem in production
- Filesystem path length limits are typically 255 characters for filenames (POSIX standard)
- Backward compatibility with existing `debug_enabled` configuration is important
- Protobuf descriptor files should be written if either Arrow or Protobuf debug is enabled (they're useful for both formats)

## Dependencies

- Existing debug file writing infrastructure (`DebugWriter` class)
- File rotation utilities (`file_rotation.rs`)
- Configuration loading system (YAML, environment variables, programmatic API)

## Clarifications

### Session 2025-12-12

- Q: Configuration validation rule - Should system require at least one debug format enabled when debug_output_dir is set, or allow both disabled? → A: Allow both disabled with output_dir set (no validation error, just no files written)
- Q: Filename length handling strategy - What should happen when rotated filename would exceed filesystem limits? → A: Use shorter naming scheme (e.g., sequential numbers instead of timestamps)
- Q: Timestamp pattern detection in base filenames - How should system detect timestamps when base filename contains underscores or timestamp-like patterns? → A: Detect timestamp pattern at end of filename only (before extension)
- Q: File retention count configuration - Should the "keep last 10 files" be fixed or configurable? → A: Configurable parameter with default of 10, allow 0 to disable retention (unlimited)
- Q: Definition of "complete" file for retention - What counts as a complete file for retention purposes? → A: Only rotated/closed files count (currently active file excluded from count)
- Q: File retention cleanup timing - When should old files be deleted to maintain retention limit? → A: Immediately after rotation when limit exceeded (delete oldest when 11th file is created)
- Q: Retention limit scope - Should retention limit apply independently per file type or combined? → A: Independent limits per file type (keep last 10 Arrow files, keep last 10 Protobuf files separately)
- Q: File deletion ordering - How should system determine which file is oldest for deletion? → A: Order by filename timestamp (parse timestamp from filename) or by sequential number if using sequential naming pattern

## Out of Scope

- Changing the debug file format or structure
- Adding new debug output formats beyond Arrow and Protobuf
- Changing the file rotation trigger logic (still based on record count and file size)
- Modifying the timestamp format used in rotated file names (still `YYYYMMDD_HHMMSS`), except when filename length constraints require shorter naming scheme

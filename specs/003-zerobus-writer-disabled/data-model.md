# Data Model: Zerobus Writer Disabled Mode

**Feature**: 003-zerobus-writer-disabled  
**Date**: 2025-12-11  
**Status**: Design Complete

## Overview

This feature adds a single configuration flag to disable Zerobus SDK transmission while maintaining debug file output. The data model changes are minimal, involving only the addition of one boolean field to the existing configuration structure.

## Entities

### WrapperConfiguration (Modified)

**Purpose**: Configuration structure for initializing ZerobusWrapper instances.

**Changes**: Add `zerobus_writer_disabled` field.

**Fields**:

| Field | Type | Default | Required | Description |
|-------|------|---------|----------|-------------|
| `zerobus_writer_disabled` | `bool` | `false` | No | When `true`, disables Zerobus SDK transmission while maintaining debug file output. When `true`, `debug_enabled` must also be `true`. |

**Relationships**:
- **WrapperConfiguration** → **ZerobusWrapper**: One-to-one (wrapper created with configuration)
- **WrapperConfiguration** → **DebugWriter**: One-to-one (optional, created if debug enabled)
- **WrapperConfiguration** → **ZerobusSdk**: Zero-to-one (created only if writer not disabled)

**Validation Rules**:
1. If `zerobus_writer_disabled` is `true`, then `debug_enabled` MUST be `true`
2. If `zerobus_writer_disabled` is `true`, then `client_id` and `client_secret` are optional (not required)
3. If `zerobus_writer_disabled` is `false` (default), existing validation rules apply

**State Transitions**:
- Configuration is immutable after creation
- No runtime state changes

**Builder Methods**:
- `with_zerobus_writer_disabled(bool) -> Self`: Sets the writer disabled flag

---

### ZerobusWrapper (No Structural Changes)

**Purpose**: Main wrapper for sending data to Zerobus.

**Changes**: No structural changes. Internal logic modified to check `zerobus_writer_disabled` flag.

**Behavioral Changes**:
- When `zerobus_writer_disabled` is `true`:
  - SDK initialization is skipped
  - Stream creation is skipped
  - Data transmission calls are skipped
  - Debug file writing continues normally
  - Returns success immediately after debug file writing

---

### TransmissionResult (No Changes)

**Purpose**: Result of a data transmission operation.

**Changes**: No structural changes. Behavior unchanged - returns success when writer is disabled and conversion succeeds.

**Fields**: (Unchanged)
- `success: bool`
- `error: Option<ZerobusError>`
- `attempts: u32`
- `latency_ms: Option<u64>`
- `batch_size_bytes: usize`

**Behavior**:
- When writer is disabled: Returns `success: true` if conversion succeeds, `success: false` if conversion fails
- No distinction needed between "skipped" and "sent" from user perspective

---

## Configuration Validation

### Validation Rules

1. **Writer Disabled Requires Debug Enabled**:
   ```rust
   if zerobus_writer_disabled && !debug_enabled {
       return Err(ZerobusError::ConfigurationError(
           "debug_enabled must be true when zerobus_writer_disabled is true".to_string()
       ));
   }
   ```

2. **Credentials Optional When Disabled**:
   - When `zerobus_writer_disabled` is `true`, `client_id` and `client_secret` validation is skipped
   - When `zerobus_writer_disabled` is `false`, existing credential validation applies

3. **Debug Output Directory**:
   - When `zerobus_writer_disabled` is `true`, `debug_output_dir` validation follows same rules as when `debug_enabled` is `true`

---

## Data Flow

### Normal Operation (Writer Enabled)

```
User → send_batch() 
  → Write Arrow debug file
  → Convert to Protobuf
  → Write Protobuf debug file
  → Initialize SDK (if needed)
  → Create stream (if needed)
  → Send to Zerobus
  → Return TransmissionResult
```

### Disabled Mode (Writer Disabled)

```
User → send_batch()
  → Write Arrow debug file
  → Convert to Protobuf
  → Write Protobuf debug file
  → Check zerobus_writer_disabled flag
  → Return success (skip SDK calls)
```

---

## Relationships

- **WrapperConfiguration** → **ZerobusWrapper**: One-to-one (wrapper created with configuration)
- **WrapperConfiguration** → **DebugWriter**: One-to-one (optional, created if debug enabled)
- **WrapperConfiguration** → **ZerobusSdk**: Zero-to-one (created only if writer not disabled and credentials provided)

## Identity & Uniqueness

- **WrapperConfiguration**: Identified by combination of endpoint, table_name, and configuration hash (unchanged)
- **ZerobusWrapper**: Single instance per configuration (unchanged)
- **TransmissionResult**: Identified by operation ID (unchanged)

## Lifecycle

1. **Configuration Creation**: User creates WrapperConfiguration with `zerobus_writer_disabled` flag
2. **Configuration Validation**: System validates that debug_enabled is true if writer_disabled is true
3. **Wrapper Initialization**: ZerobusWrapper created from configuration
   - If writer disabled: SDK initialization skipped
   - If writer enabled: SDK initialized (existing behavior)
4. **Data Transmission**: 
   - If writer disabled: Debug files written, return success immediately
   - If writer enabled: Normal transmission flow (existing behavior)

## Data Volume Assumptions

- No changes to data volume assumptions
- Debug file writing behavior unchanged
- No network traffic when writer disabled

## Notes

- This feature adds minimal complexity to the data model
- No new entities required
- Single boolean field addition
- Backward compatible (default value is false)
- Configuration is immutable after creation


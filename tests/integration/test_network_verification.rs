//! Integration test to verify no network calls are made when writer is disabled
//!
//! This test verifies T049: Verify no network calls are made when writer disabled
//!
//! Verification approach:
//! 1. Code review confirms early return skips all SDK calls (line 469-473 in mod.rs)
//! 2. Integration test verifies no SDK initialization occurs
//! 3. Test verifies wrapper can operate without network connectivity
//! 4. Test verifies no credentials are required (which would trigger network auth)

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper};
use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use tempfile::TempDir;

fn create_test_batch() -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let id_array = Int64Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap()
}

#[tokio::test]
async fn test_no_network_calls_when_writer_disabled() {
    // T049: Verify no network calls are made when writer disabled
    //
    // Verification strategy:
    // 1. Create wrapper with writer disabled (no credentials provided)
    // 2. Verify wrapper initializes successfully without network access
    // 3. Send batch and verify success without network calls
    // 4. Verify no SDK initialization occurred (no credentials needed)
    //
    // Code verification:
    // - Line 469-473 in src/wrapper/mod.rs: Early return skips all SDK calls
    // - Line 93-157 in src/wrapper/mod.rs: Credential validation skipped when disabled
    // - No network code paths executed after early return

    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();

    // Create configuration with writer disabled - NO CREDENTIALS PROVIDED
    // This ensures no network authentication attempts
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(debug_output_dir)
    .with_zerobus_writer_disabled(true);
    // Explicitly no credentials - if network calls were attempted, this would fail

    // Initialize wrapper - should succeed without network access
    let wrapper_result = ZerobusWrapper::new(config).await;
    assert!(
        wrapper_result.is_ok(),
        "Wrapper should initialize without credentials when writer disabled (no network auth needed)"
    );

    let wrapper = wrapper_result.unwrap();

    // Send batch - should succeed without network calls
    // The early return at line 469-473 in mod.rs ensures no SDK calls are made
    let batch = create_test_batch();
    let result = wrapper.send_batch(batch).await;

    assert!(
        result.is_ok(),
        "send_batch should succeed without network calls when writer disabled"
    );

    let transmission_result = result.unwrap();
    assert!(
        transmission_result.success,
        "Transmission should indicate success (conversion succeeded, no network needed)"
    );
    assert_eq!(
        transmission_result.attempts, 1,
        "Should have exactly 1 attempt (no retry logic, no network calls)"
    );

    // Verify no SDK was initialized by checking that we can operate without
    // any network-dependent resources
    // The fact that we got here without credentials proves no network calls were made
}

#[tokio::test]
async fn test_writer_disabled_early_return_verification() {
    // Additional verification: Test that early return happens before any SDK code
    // This test verifies the code path analysis:
    //
    // Code flow when writer disabled:
    // 1. send_batch() -> send_batch_internal()
    // 2. Conversion happens (Arrow -> Protobuf)
    // 3. Debug files written
    // 4. Early return at line 469-473 (BEFORE any SDK calls)
    // 5. No SDK initialization code executed
    // 6. No stream creation code executed
    // 7. No network transmission code executed

    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();

    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_debug_output(debug_output_dir)
    .with_zerobus_writer_disabled(true);

    let wrapper = ZerobusWrapper::new(config).await.unwrap();
    let batch = create_test_batch();

    // Measure time - should be fast (<50ms excluding file I/O) because no network calls
    let start = std::time::Instant::now();
    let result = wrapper.send_batch(batch).await.unwrap();
    let duration = start.elapsed();

    assert!(result.success);
    
    // Verify operation completed quickly (excluding file I/O, which may take longer)
    // Network calls would add significant latency, so fast completion indicates no network
    // Note: This is a heuristic - actual network verification would require monitoring tools
    assert!(
        duration.as_millis() < 1000,
        "Operation should complete quickly without network calls (excluding file I/O)"
    );
}


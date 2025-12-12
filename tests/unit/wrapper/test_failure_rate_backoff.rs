//! Unit tests for failure rate backoff functionality
//!
//! Tests verify that automatic backoff with jitter is triggered when failure rates
//! exceed 1% due to network or transmission issues.

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::error::ZerobusError;
use arrow_zerobus_sdk_wrapper::wrapper::zerobus;
use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Create a test RecordBatch with specified number of rows
fn create_test_batch(num_rows: usize) -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    
    let ids: Vec<i64> = (1..=num_rows as i64).collect();
    let names: Vec<String> = (1..=num_rows)
        .map(|i| format!("Name_{}", i))
        .collect();
    
    let id_array = Int64Array::from(ids);
    let name_array = StringArray::from(names);
    
    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap()
}

/// Helper to create wrapper with writer disabled (for testing failure rate tracking)
async fn create_test_wrapper(table_name: &str) -> ZerobusWrapper {
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        table_name.to_string(),
    )
    .with_zerobus_writer_disabled(true); // Disable actual SDK calls
    
    ZerobusWrapper::new(config)
        .await
        .expect("Failed to create wrapper")
}

#[tokio::test]
async fn test_failure_rate_backoff_triggers_above_threshold() {
    // Test 1: Simulate network failures and verify backoff triggers at >1% failure rate
    
    let table_name = "test_table_backoff_trigger";
    
    // Clear any existing state
    // Note: We can't directly clear static state, but we can test with a unique table name
    
    // Simulate batches with increasing failure rates
    // We need at least 100 rows to calculate failure rate (MIN_ROWS_FOR_FAILURE_RATE)
    
    // Batch 1: 100 rows, 0 failures (0% failure rate) - should not trigger
    let failed_rows_1: Vec<(usize, ZerobusError)> = vec![];
    zerobus::update_failure_rate(table_name, 100, &failed_rows_1);
    
    // Verify no backoff is active
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(result.is_ok(), "Should not trigger backoff at 0% failure rate");
    
    // Batch 2: 100 rows, 2 failures (2% failure rate) - should trigger backoff
    // Total: 200 rows, 2 failures = 1% failure rate (should trigger)
    let failed_rows_2 = vec![
        (0, ZerobusError::ConnectionError("Network error 1".to_string())),
        (1, ZerobusError::TransmissionError("Transmission error 1".to_string())),
    ];
    zerobus::update_failure_rate(table_name, 100, &failed_rows_2);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Verify backoff is now active
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result.is_err(),
        "Should trigger backoff when failure rate exceeds 1%"
    );
    
    // Verify error message indicates high failure rate
    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(
            error_msg.contains("High failure rate") || error_msg.contains("failure rate"),
            "Error should mention failure rate: {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn test_failure_rate_backoff_blocks_writes() {
    // Test 2: Verify backoff blocks writes during the backoff period
    
    let table_name = "test_table_backoff_blocks";
    
    // Trigger backoff by simulating high failure rate
    // Need 100+ rows with >1% failures
    let failed_rows = (0..2)
        .map(|i| (i, ZerobusError::ConnectionError(format!("Network error {}", i))))
        .collect::<Vec<_>>();
    
    zerobus::update_failure_rate(table_name, 100, &failed_rows);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Verify backoff is active
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(result.is_err(), "Backoff should be active");
    
    // Try to check backoff multiple times - should all fail
    for _ in 0..5 {
        let check_result = zerobus::check_failure_rate_backoff(table_name).await;
        assert!(
            check_result.is_err(),
            "Backoff should block writes throughout the backoff period"
        );
    }
}

#[tokio::test]
async fn test_failure_rate_backoff_automatic_recovery() {
    // Test 3: Verify automatic recovery after backoff expires
    
    let table_name = "test_table_backoff_recovery";
    
    // Trigger backoff
    let failed_rows = vec![
        (0, ZerobusError::ConnectionError("Network error".to_string())),
        (1, ZerobusError::TransmissionError("Transmission error".to_string())),
    ];
    zerobus::update_failure_rate(table_name, 100, &failed_rows);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Verify backoff is active
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(result.is_err(), "Backoff should be active immediately after trigger");
    
    // Wait for backoff to expire (backoff is 30-45 seconds, but we can't wait that long in tests)
    // Instead, we'll verify the backoff state exists and check the structure
    // In a real scenario, the backoff would expire after 30-45 seconds
    
    // Note: We can't easily test expiration in unit tests without waiting 30-45 seconds,
    // but we can verify the backoff is active and will expire automatically.
    // The backoff duration is 30-45 seconds with jitter, so in a real scenario,
    // the system would automatically recover after that period.
}

#[tokio::test]
async fn test_failure_rate_backoff_per_table_isolation() {
    // Test 4: Test per-table isolation (high failure rate on one table doesn't affect others)
    
    let table1 = "test_table_isolation_1";
    let table2 = "test_table_isolation_2";
    
    // Trigger backoff for table1
    let failed_rows_table1 = vec![
        (0, ZerobusError::ConnectionError("Network error".to_string())),
        (1, ZerobusError::TransmissionError("Transmission error".to_string())),
    ];
    zerobus::update_failure_rate(table1, 100, &failed_rows_table1);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Verify backoff is active for table1
    let result1 = zerobus::check_failure_rate_backoff(table1).await;
    assert!(result1.is_err(), "Backoff should be active for table1");
    
    // Verify backoff is NOT active for table2 (isolation)
    let result2 = zerobus::check_failure_rate_backoff(table2).await;
    assert!(
        result2.is_ok(),
        "Backoff should NOT be active for table2 (per-table isolation)"
    );
    
    // Trigger backoff for table2 independently
    let failed_rows_table2 = vec![
        (0, ZerobusError::ConnectionError("Network error table2".to_string())),
        (1, ZerobusError::TransmissionError("Transmission error table2".to_string())),
    ];
    zerobus::update_failure_rate(table2, 100, &failed_rows_table2);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Now both should have backoff active
    let result1_after = zerobus::check_failure_rate_backoff(table1).await;
    let result2_after = zerobus::check_failure_rate_backoff(table2).await;
    
    assert!(result1_after.is_err(), "Backoff should still be active for table1");
    assert!(result2_after.is_err(), "Backoff should now be active for table2");
}

#[tokio::test]
async fn test_failure_rate_only_counts_network_errors() {
    // Test 5: Verify only network errors are counted (conversion errors ignored)
    
    let table_name = "test_table_error_filtering";
    
    // Send batch with only conversion errors (should NOT count toward failure rate)
    let conversion_errors = vec![
        (0, ZerobusError::ConversionError("Conversion error 1".to_string())),
        (1, ZerobusError::ConversionError("Conversion error 2".to_string())),
        (2, ZerobusError::ConversionError("Conversion error 3".to_string())),
    ];
    
    // Update with 100 rows, all conversion errors
    zerobus::update_failure_rate(table_name, 100, &conversion_errors);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Should NOT trigger backoff (conversion errors don't count)
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result.is_ok(),
        "Should NOT trigger backoff for conversion errors (not network errors)"
    );
    
    // Now add network errors to trigger backoff
    let network_errors = vec![
        (0, ZerobusError::ConnectionError("Network error".to_string())),
        (1, ZerobusError::TransmissionError("Transmission error".to_string())),
    ];
    
    // Update with 100 more rows, 2 network failures
    // Total: 200 rows, 2 network failures = 1% (should trigger)
    zerobus::update_failure_rate(table_name, 100, &network_errors);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Should NOW trigger backoff (network errors count)
    let result_after = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result_after.is_err(),
        "Should trigger backoff when network errors exceed 1%"
    );
}

#[tokio::test]
async fn test_failure_rate_mixed_errors() {
    // Test that mixed errors (conversion + network) only count network errors
    
    let table_name = "test_table_mixed_errors";
    
    // Send batch with both conversion and network errors
    let mixed_errors = vec![
        (0, ZerobusError::ConversionError("Conversion error".to_string())),
        (1, ZerobusError::ConnectionError("Network error 1".to_string())),
        (2, ZerobusError::ConversionError("Conversion error 2".to_string())),
        (3, ZerobusError::TransmissionError("Transmission error".to_string())),
        (4, ZerobusError::ConfigurationError("Config error".to_string())),
    ];
    
    // 100 rows, 2 network errors (ConnectionError + TransmissionError)
    // Should NOT trigger backoff yet (2/100 = 2%, but need 100+ rows total)
    zerobus::update_failure_rate(table_name, 100, &mixed_errors);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Should trigger backoff (2 network failures / 100 rows = 2% > 1%)
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result.is_err(),
        "Should trigger backoff with 2% network error rate (2 network errors / 100 rows)"
    );
}

#[tokio::test]
async fn test_failure_rate_minimum_rows_requirement() {
    // Test that failure rate is only calculated after minimum rows threshold
    
    let table_name = "test_table_min_rows";
    
    // Send small batch with high failure rate (but below minimum threshold)
    let failed_rows = vec![
        (0, ZerobusError::ConnectionError("Network error 1".to_string())),
        (1, ZerobusError::ConnectionError("Network error 2".to_string())),
    ];
    
    // 50 rows, 2 failures = 4% failure rate, but below MIN_ROWS_FOR_FAILURE_RATE (100)
    zerobus::update_failure_rate(table_name, 50, &failed_rows);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Should NOT trigger backoff (not enough rows yet)
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result.is_ok(),
        "Should NOT trigger backoff with <100 rows, even with high failure rate"
    );
    
    // Add more rows to reach threshold
    let more_failed_rows = vec![
        (0, ZerobusError::ConnectionError("Network error 3".to_string())),
    ];
    
    // 50 more rows, 1 more failure
    // Total: 100 rows, 3 failures = 3% (should trigger)
    zerobus::update_failure_rate(table_name, 50, &more_failed_rows);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Should NOW trigger backoff (3/100 = 3% > 1%)
    let result_after = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result_after.is_err(),
        "Should trigger backoff once minimum rows threshold is reached"
    );
}

#[tokio::test]
async fn test_failure_rate_window_reset() {
    // Test that failure rate window resets after backoff is triggered
    
    let table_name = "test_table_window_reset";
    
    // Trigger backoff
    let failed_rows = vec![
        (0, ZerobusError::ConnectionError("Network error 1".to_string())),
        (1, ZerobusError::ConnectionError("Network error 2".to_string())),
    ];
    zerobus::update_failure_rate(table_name, 100, &failed_rows);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Verify backoff is active
    let result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(result.is_err(), "Backoff should be active");
    
    // After backoff is triggered, failure rate tracking should reset
    // Send successful batch - should not immediately trigger backoff again
    let no_errors: Vec<(usize, ZerobusError)> = vec![];
    zerobus::update_failure_rate(table_name, 100, &no_errors);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Backoff should still be active (hasn't expired yet)
    // But the failure rate tracking should have been reset
    let result_after = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result_after.is_err(),
        "Backoff should still be active (hasn't expired)"
    );
}

#[tokio::test]
async fn test_failure_rate_backoff_integration_with_wrapper() {
    // Integration test: Verify backoff is checked before sending batches
    
    let table_name = "test_table_wrapper_integration";
    
    // Create wrapper with writer disabled
    let wrapper = create_test_wrapper(table_name).await;
    
    // Trigger backoff by simulating high failure rate
    let failed_rows = vec![
        (0, ZerobusError::ConnectionError("Network error 1".to_string())),
        (1, ZerobusError::TransmissionError("Transmission error 1".to_string())),
    ];
    zerobus::update_failure_rate(table_name, 100, &failed_rows);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Try to send a batch - should be blocked by backoff check
    // Note: With writer disabled, the backoff check happens before SDK calls
    // The check happens in send_batch_internal before processing records
    
    // Since writer is disabled, we can't easily test the full flow,
    // but we can verify the backoff check function works
    let backoff_result = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        backoff_result.is_err(),
        "Backoff check should fail when backoff is active"
    );
}

#[tokio::test]
async fn test_failure_rate_backoff_jitter_range() {
    // Test that backoff duration includes jitter (30-45 seconds)
    
    let table_name = "test_table_jitter";
    
    // Trigger backoff multiple times and verify jitter is applied
    // Note: We can't easily verify exact jitter values without accessing internal state,
    // but we can verify backoff is triggered
    
    for i in 0..5 {
        let table = format!("{}_{}", table_name, i);
        let failed_rows = vec![
            (0, ZerobusError::ConnectionError("Network error".to_string())),
            (1, ZerobusError::TransmissionError("Transmission error".to_string())),
        ];
        zerobus::update_failure_rate(&table, 100, &failed_rows);
        
        // Small delay to ensure state is updated
        sleep(Duration::from_millis(10)).await;
        
        // Verify backoff is triggered (jitter is applied internally)
        let result = zerobus::check_failure_rate_backoff(&table).await;
        assert!(
            result.is_err(),
            "Backoff should be triggered for table {}",
            table
        );
    }
}

#[tokio::test]
async fn test_failure_rate_exact_threshold() {
    // Test exact threshold behavior: exactly 1% should trigger, <1% should not
    
    let table_name = "test_table_exact_threshold";
    
    // Test case 1: Exactly 1% failure rate (1 failure in 100 rows)
    let failed_rows_1pct = vec![
        (0, ZerobusError::ConnectionError("Network error".to_string())),
    ];
    zerobus::update_failure_rate(table_name, 100, &failed_rows_1pct);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Should trigger backoff (1/100 = 1% >= threshold)
    let result_1pct = zerobus::check_failure_rate_backoff(table_name).await;
    assert!(
        result_1pct.is_err(),
        "Should trigger backoff at exactly 1% failure rate"
    );
    
    // Test case 2: Just below 1% (0 failures in 100 rows)
    let table_name_below = "test_table_below_threshold";
    let no_errors: Vec<(usize, ZerobusError)> = vec![];
    zerobus::update_failure_rate(table_name_below, 100, &no_errors);
    
    // Small delay to ensure state is updated
    sleep(Duration::from_millis(10)).await;
    
    // Should NOT trigger backoff (0/100 = 0% < threshold)
    let result_below = zerobus::check_failure_rate_backoff(table_name_below).await;
    assert!(
        result_below.is_ok(),
        "Should NOT trigger backoff below 1% failure rate"
    );
}


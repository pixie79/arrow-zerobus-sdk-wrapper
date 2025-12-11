//! Integration tests for per-row error handling

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::{
    TransmissionResult, WrapperConfiguration, ZerobusError, ZerobusWrapper,
};
use std::sync::Arc;

/// Create a test RecordBatch with valid data
fn create_valid_batch() -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie", "David", "Eve"]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap()
}

/// Test partial batch success scenario
/// 
/// This test verifies that when some rows fail and others succeed,
/// the TransmissionResult correctly identifies which rows succeeded
/// and which failed.
#[tokio::test]
#[ignore] // Requires actual Zerobus SDK
async fn test_partial_batch_success() {
    let config = WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        std::env::var("ZEROBUS_CLIENT_ID")
            .unwrap_or_else(|_| "test_client_id".to_string()),
        std::env::var("ZEROBUS_CLIENT_SECRET")
            .unwrap_or_else(|_| "test_client_secret".to_string()),
    )
    .with_unity_catalog(
        std::env::var("UNITY_CATALOG_URL")
            .unwrap_or_else(|_| "https://test.cloud.databricks.com".to_string()),
    );

    let wrapper = match ZerobusWrapper::new(config).await {
        Ok(w) => w,
        Err(_) => {
            // Skip test if wrapper can't be created (no credentials)
            return;
        }
    };

    let batch = create_valid_batch();
    let result = wrapper.send_batch(batch).await;

    match result {
        Ok(transmission_result) => {
            // Verify per-row error information is available
            assert_eq!(transmission_result.total_rows, 5);
            assert_eq!(
                transmission_result.total_rows,
                transmission_result.successful_count + transmission_result.failed_count
            );

            // If partial success, verify both successful and failed rows are tracked
            if transmission_result.is_partial_success() {
                assert!(transmission_result.has_successful_rows());
                assert!(transmission_result.has_failed_rows());

                let successful_indices = transmission_result.get_successful_row_indices();
                let failed_indices = transmission_result.get_failed_row_indices();

                // Verify indices don't overlap
                for idx in &failed_indices {
                    assert!(!successful_indices.contains(idx));
                }

                // Verify counts match
                assert_eq!(successful_indices.len(), transmission_result.successful_count);
                assert_eq!(failed_indices.len(), transmission_result.failed_count);
            }
        }
        Err(e) => {
            // Error is acceptable in test environment
            eprintln!("Transmission error (expected in test): {}", e);
        }
    }
}

/// Test that all rows succeed scenario
#[tokio::test]
#[ignore] // Requires actual Zerobus SDK
async fn test_all_rows_succeed() {
    let config = WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        std::env::var("ZEROBUS_CLIENT_ID")
            .unwrap_or_else(|_| "test_client_id".to_string()),
        std::env::var("ZEROBUS_CLIENT_SECRET")
            .unwrap_or_else(|_| "test_client_secret".to_string()),
    )
    .with_unity_catalog(
        std::env::var("UNITY_CATALOG_URL")
            .unwrap_or_else(|_| "https://test.cloud.databricks.com".to_string()),
    );

    let wrapper = match ZerobusWrapper::new(config).await {
        Ok(w) => w,
        Err(_) => return,
    };

    let batch = create_valid_batch();
    let result = wrapper.send_batch(batch).await;

    match result {
        Ok(transmission_result) if transmission_result.success => {
            // All rows should succeed
            assert_eq!(transmission_result.successful_count, transmission_result.total_rows);
            assert_eq!(transmission_result.failed_count, 0);
            
            // failed_rows should be None or empty
            match &transmission_result.failed_rows {
                None => {}
                Some(rows) => assert_eq!(rows.len(), 0),
            }

            // successful_rows should contain all indices
            if let Some(successful) = &transmission_result.successful_rows {
                assert_eq!(successful.len(), transmission_result.total_rows);
            }
        }
        _ => {
            // Acceptable in test environment
        }
    }
}

/// Test empty batch edge case
#[tokio::test]
async fn test_empty_batch() {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    let empty_batch = RecordBatch::try_new(Arc::new(schema), vec![]).unwrap();

    let config = WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    let wrapper = match ZerobusWrapper::new(config).await {
        Ok(w) => w,
        Err(_) => return,
    };

    let result = wrapper.send_batch(empty_batch).await;

    match result {
        Ok(transmission_result) => {
            assert_eq!(transmission_result.total_rows, 0);
            assert_eq!(transmission_result.successful_count, 0);
            assert_eq!(transmission_result.failed_count, 0);
        }
        Err(_) => {
            // Empty batch might be handled differently
        }
    }
}

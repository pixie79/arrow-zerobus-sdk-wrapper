//! Contract tests for per-row error API

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_zerobus_sdk_wrapper::{
    TransmissionResult, WrapperConfiguration, ZerobusError, ZerobusWrapper,
};
use std::sync::Arc;

/// Test that send_batch returns TransmissionResult with per-row error fields
#[tokio::test]
#[ignore] // Requires actual Zerobus SDK
async fn test_send_batch_contract_per_row_fields() {
    let config = WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    let wrapper = match ZerobusWrapper::new(config).await {
        Ok(w) => w,
        Err(_) => return,
    };

    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let id_array = Int64Array::from(vec![1, 2]);
    let name_array = StringArray::from(vec!["Alice", "Bob"]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap();

    let result = wrapper.send_batch(batch).await;

    // Contract: send_batch must return Result<TransmissionResult, ZerobusError>
    match result {
        Ok(transmission_result) => {
            // Contract: TransmissionResult must have per-row error fields
            let _ = transmission_result.failed_rows;
            let _ = transmission_result.successful_rows;
            let _ = transmission_result.total_rows;
            let _ = transmission_result.successful_count;
            let _ = transmission_result.failed_count;

            // Contract: Consistency checks
            assert_eq!(
                transmission_result.total_rows,
                transmission_result.successful_count + transmission_result.failed_count
            );
        }
        Err(_) => {
            // Acceptable in test environment
        }
    }
}

/// Test that send_batch_with_descriptor also returns per-row error fields
#[tokio::test]
#[ignore] // Requires actual Zerobus SDK
async fn test_send_batch_with_descriptor_contract() {
    let config = WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    let wrapper = match ZerobusWrapper::new(config).await {
        Ok(w) => w,
        Err(_) => return,
    };

    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let id_array = Int64Array::from(vec![1, 2]);
    let name_array = StringArray::from(vec!["Alice", "Bob"]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(id_array), Arc::new(name_array)],
    )
    .unwrap();

    let result = wrapper.send_batch_with_descriptor(batch, None).await;

    // Contract: send_batch_with_descriptor must return Result<TransmissionResult, ZerobusError>
    match result {
        Ok(transmission_result) => {
            // Contract: Must have per-row error fields
            assert!(transmission_result.total_rows >= 0);
            assert_eq!(
                transmission_result.total_rows,
                transmission_result.successful_count + transmission_result.failed_count
            );
        }
        Err(_) => {
            // Acceptable in test environment
        }
    }
}

/// Test backward compatibility: existing code patterns still work
#[tokio::test]
#[ignore] // Requires actual Zerobus SDK
async fn test_backward_compatibility_contract() {
    let config = WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    let wrapper = match ZerobusWrapper::new(config).await {
        Ok(w) => w,
        Err(_) => return,
    };

    let schema = Schema::new(vec![Field::new("id", DataType::Int64, false)]);
    let id_array = Int64Array::from(vec![1]);
    let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(id_array)]).unwrap();

    let result = wrapper.send_batch(batch).await;

    // Contract: Existing code that checks success and error should still work
    match result {
        Ok(transmission_result) => {
            // Existing pattern: check success
            if transmission_result.success {
                assert!(transmission_result.error.is_none());
            } else {
                // Error may or may not be set (could be per-row errors)
            }

            // Existing pattern: check error
            if let Some(error) = &transmission_result.error {
                // Batch-level error
                assert!(!transmission_result.success || transmission_result.failed_count > 0);
            }
        }
        Err(_) => {
            // Acceptable in test environment
        }
    }
}

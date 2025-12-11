//! Integration tests for quickstart validation
//!
//! These tests validate that the examples in quickstart.md work correctly

use arrow_zerobus_sdk_wrapper::{WrapperConfiguration, ZerobusWrapper, ZerobusError};
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
async fn test_quickstart_basic_usage() {
    // Test basic usage example from quickstart.md
    let temp_dir = TempDir::new().unwrap();
    let debug_output_dir = temp_dir.path().to_path_buf();

    let config = WrapperConfiguration::new(
        "https://workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    )
    .with_debug_output(debug_output_dir)
    .with_zerobus_writer_disabled(true);

    // Validate configuration
    assert!(config.validate().is_ok(), "Configuration should be valid");

    // Initialize wrapper (should succeed without credentials)
    let wrapper = ZerobusWrapper::new(config).await.unwrap();

    // Create test batch
    let batch = create_test_batch();

    // Send batch - writes debug files but skips network calls
    let result = wrapper.send_batch(batch).await.unwrap();
    assert!(result.success, "send_batch should succeed when writer disabled");

    // Flush to ensure files are written
    wrapper.flush().await.unwrap();
}

#[tokio::test]
async fn test_quickstart_configuration_validation() {
    // Test configuration validation example from quickstart.md
    let config = WrapperConfiguration::new(
        "https://workspace.cloud.databricks.com".to_string(),
        "my_table".to_string(),
    )
    .with_zerobus_writer_disabled(true); // But debug_enabled is false (default)

    // Validation should fail
    match config.validate() {
        Err(ZerobusError::ConfigurationError(msg)) => {
            assert!(
                msg.contains("debug_enabled must be true"),
                "Error message should mention debug_enabled requirement"
            );
        }
        Ok(_) => panic!("Validation should fail when writer disabled but debug not enabled"),
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }
}

#[tokio::test]
async fn test_quickstart_unit_testing_example() {
    // Test unit testing example from quickstart.md
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
    let result = wrapper.send_batch(batch).await.unwrap();
    assert!(result.success, "Conversion should succeed");
}


//! Common test utilities and mocks
//!
//! This module provides shared test infrastructure for all test modules.

mod mocks;

pub use mocks::*;

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Create a test Arrow RecordBatch
///
/// Returns a simple RecordBatch with two columns (id: Int64, name: String)
/// containing 3 rows of test data.
pub fn create_test_record_batch() -> RecordBatch {
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
    .expect("Failed to create test RecordBatch")
}

/// Create a test configuration
///
/// Returns a WrapperConfiguration with test values.
pub fn create_test_config() -> crate::WrapperConfiguration {
    crate::WrapperConfiguration::new(
        "https://test-workspace.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials("test_client_id".to_string(), "test_client_secret".to_string())
    .with_unity_catalog("https://test-unity-catalog-url".to_string())
}


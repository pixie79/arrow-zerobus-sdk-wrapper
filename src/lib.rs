//! Arrow Zerobus SDK Wrapper
//!
//! Cross-platform Rust SDK wrapper for Databricks Zerobus with Python bindings.
//! Provides a unified API for sending Arrow RecordBatch data to Zerobus with
//! automatic protocol conversion, authentication, retry logic, and observability.
//!
//! # Features
//!
//! - Rust SDK API for sending Arrow RecordBatch data to Zerobus
//! - Python bindings (Python 3.11+) via PyO3
//! - Automatic retry with exponential backoff + jitter
//! - Automatic token refresh for long-running operations
//! - OpenTelemetry observability integration
//! - Optional debug file output (Arrow + Protobuf)
//! - Thread-safe concurrent operations
//!
//! # Example
//!
//! ```no_run
//! use arrow_zerobus_sdk_wrapper::{ZerobusWrapper, WrapperConfiguration};
//! use arrow::record_batch::RecordBatch;
//!
//! # async fn example() -> Result<(), arrow_zerobus_sdk_wrapper::ZerobusError> {
//! # let config = WrapperConfiguration::new(
//! #     "https://workspace.cloud.databricks.com".to_string(),
//! #     "my_table".to_string(),
//! # )
//! # .with_credentials("client_id".to_string(), "client_secret".to_string())
//! # .with_unity_catalog("https://unity-catalog-url".to_string());
//! # let wrapper = ZerobusWrapper::new(config).await?;
//! # // Create and send a RecordBatch here
//! # wrapper.shutdown().await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
pub mod observability;
pub mod utils;
pub mod wrapper;

#[cfg(feature = "python")]
pub mod python;

pub use config::{OtlpConfig, OtlpSdkConfig, WrapperConfiguration};
pub use error::ZerobusError;
pub use wrapper::{ErrorStatistics, TransmissionResult, ZerobusWrapper};

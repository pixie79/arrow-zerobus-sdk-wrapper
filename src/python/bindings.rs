//! PyO3 bindings implementation
//!
//! This module implements Python bindings for the Zerobus SDK Wrapper,
//! providing a Pythonic API that matches the Rust API functionality.

// PyO3's #[pymethods] macro generates non-local impl blocks, which is necessary for bindings
// This lint must be disabled for PyO3 bindings to work correctly
#![allow(non_local_definitions)]

use crate::config::OtlpSdkConfig;
use crate::config::WrapperConfiguration;
use crate::error::ZerobusError;
use crate::wrapper::{TransmissionResult, ZerobusWrapper};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use pyo3::exceptions::{PyException, PyNotImplementedError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Register all Python classes and functions in the module
pub fn register_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyZerobusWrapper>()?;
    m.add_class::<PyTransmissionResult>()?;
    m.add_class::<PyWrapperConfiguration>()?;

    // Register exception classes - base class must be registered first
    m.add_class::<PyZerobusError>()?;
    m.add_class::<PyConfigurationError>()?;
    m.add_class::<PyAuthenticationError>()?;
    m.add_class::<PyConnectionError>()?;
    m.add_class::<PyConversionError>()?;
    m.add_class::<PyTransmissionError>()?;
    m.add_class::<PyRetryExhausted>()?;
    m.add_class::<PyTokenRefreshError>()?;

    Ok(())
}

/// Convert Rust ZerobusError to Python exception
// Note: Made pub for re-export to tests (which are in a separate crate)
pub fn rust_error_to_python_error(error: ZerobusError) -> PyErr {
    match error {
        ZerobusError::ConfigurationError(msg) => PyErr::new::<PyConfigurationError, _>(msg),
        ZerobusError::AuthenticationError(msg) => PyErr::new::<PyAuthenticationError, _>(msg),
        ZerobusError::ConnectionError(msg) => PyErr::new::<PyConnectionError, _>(msg),
        ZerobusError::ConversionError(msg) => PyErr::new::<PyConversionError, _>(msg),
        ZerobusError::TransmissionError(msg) => PyErr::new::<PyTransmissionError, _>(msg),
        ZerobusError::RetryExhausted(msg) => PyErr::new::<PyRetryExhausted, _>(msg),
        ZerobusError::TokenRefreshError(msg) => PyErr::new::<PyTokenRefreshError, _>(msg),
    }
}

// Exception classes
// Note: In PyO3, all custom exceptions must extend PyException directly.
// We cannot use a custom base class (PyZerobusError) for other exceptions
// because PyO3 doesn't support that pattern. Instead, all exceptions extend
// PyException directly, but they're logically grouped as ZerobusError exceptions.
#[pyclass(name = "ZerobusError", extends=PyException)]
#[derive(Debug)]
pub struct PyZerobusError;

#[pymethods]
impl PyZerobusError {
    // Base exception class for Zerobus errors
}

// Exception classes with message storage for Python construction
#[pyclass(name = "ConfigurationError", extends=PyException)]
#[derive(Debug)]
pub struct PyConfigurationError {
    message: String,
}

#[pyclass(name = "AuthenticationError", extends=PyException)]
#[derive(Debug)]
pub struct PyAuthenticationError {
    message: String,
}

#[pyclass(name = "ConnectionError", extends=PyException)]
#[derive(Debug)]
pub struct PyConnectionError {
    message: String,
}

#[pyclass(name = "ConversionError", extends=PyException)]
#[derive(Debug)]
pub struct PyConversionError {
    message: String,
}

#[pyclass(name = "TransmissionError", extends=PyException)]
#[derive(Debug)]
pub struct PyTransmissionError {
    message: String,
}

#[pyclass(name = "RetryExhausted", extends=PyException)]
#[derive(Debug)]
pub struct PyRetryExhausted {
    message: String,
}

#[pyclass(name = "TokenRefreshError", extends=PyException)]
#[derive(Debug)]
pub struct PyTokenRefreshError {
    message: String,
}

// Internal helper methods for creating PyErr from Rust
// These are used by rust_error_to_python_error to convert Rust errors to Python exceptions
#[allow(dead_code)] // Used indirectly via rust_error_to_python_error
impl PyConfigurationError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyConfigurationError, _>(msg)
    }
}

#[allow(dead_code)] // Used indirectly via rust_error_to_python_error
impl PyAuthenticationError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyAuthenticationError, _>(msg)
    }
}

#[allow(dead_code)] // Used indirectly via rust_error_to_python_error
impl PyConnectionError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyConnectionError, _>(msg)
    }
}

#[allow(dead_code)] // Used indirectly via rust_error_to_python_error
impl PyConversionError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyConversionError, _>(msg)
    }
}

#[allow(dead_code)] // Used indirectly via rust_error_to_python_error
impl PyTransmissionError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyTransmissionError, _>(msg)
    }
}

#[allow(dead_code)] // Used indirectly via rust_error_to_python_error
impl PyRetryExhausted {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyRetryExhausted, _>(msg)
    }
}

#[allow(dead_code)] // Used indirectly via rust_error_to_python_error
impl PyTokenRefreshError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyTokenRefreshError, _>(msg)
    }
}

// Python constructors for error classes
#[pymethods]
impl PyConfigurationError {
    #[new]
    fn new(msg: String) -> Self {
        Self { message: msg }
    }

    fn __str__(&self) -> &str {
        &self.message
    }
}

#[pymethods]
impl PyAuthenticationError {
    #[new]
    fn new(msg: String) -> Self {
        Self { message: msg }
    }

    fn __str__(&self) -> &str {
        &self.message
    }
}

#[pymethods]
impl PyConnectionError {
    #[new]
    fn new(msg: String) -> Self {
        Self { message: msg }
    }

    fn __str__(&self) -> &str {
        &self.message
    }
}

#[pymethods]
impl PyConversionError {
    #[new]
    fn new(msg: String) -> Self {
        Self { message: msg }
    }

    fn __str__(&self) -> &str {
        &self.message
    }
}

#[pymethods]
impl PyTransmissionError {
    #[new]
    fn new(msg: String) -> Self {
        Self { message: msg }
    }

    fn __str__(&self) -> &str {
        &self.message
    }
}

#[pymethods]
impl PyRetryExhausted {
    #[new]
    fn new(msg: String) -> Self {
        Self { message: msg }
    }

    fn __str__(&self) -> &str {
        &self.message
    }
}

#[pymethods]
impl PyTokenRefreshError {
    #[new]
    fn new(msg: String) -> Self {
        Self { message: msg }
    }

    fn __str__(&self) -> &str {
        &self.message
    }
}

/// Python wrapper for WrapperConfiguration
#[pyclass(name = "WrapperConfiguration")]
#[derive(Clone)]
#[allow(non_local_definitions)]
pub struct PyWrapperConfiguration {
    inner: WrapperConfiguration,
}

#[pymethods]
#[allow(clippy::too_many_arguments)]
impl PyWrapperConfiguration {
    /// Initialize WrapperConfiguration with parameters.
    ///
    /// Args:
    ///     endpoint: Zerobus endpoint URL (required)
    ///     table_name: Target table name (required)
    ///     client_id: OAuth2 client ID (optional when zerobus_writer_disabled is True)
    ///     client_secret: OAuth2 client secret (optional when zerobus_writer_disabled is True)
    ///     unity_catalog_url: Unity Catalog URL (optional when zerobus_writer_disabled is True)
    ///     observability_enabled: Enable OpenTelemetry observability
    ///     observability_config: OpenTelemetry configuration dict
    ///     debug_enabled: Enable debug file output (required when zerobus_writer_disabled is True)
    ///     debug_output_dir: Output directory for debug files (required when debug_enabled is True)
    ///     debug_flush_interval_secs: Debug file flush interval in seconds
    ///     debug_max_file_size: Maximum debug file size before rotation
    ///     retry_max_attempts: Maximum retry attempts for transient failures
    ///     retry_base_delay_ms: Base delay in milliseconds for exponential backoff
    ///     retry_max_delay_ms: Maximum delay in milliseconds for exponential backoff
    ///     zerobus_writer_disabled: Disable Zerobus SDK transmission while maintaining debug output
    ///
    /// Raises:
    ///     ZerobusError: If configuration is invalid or initialization fails
    ///         - ConfigurationError if debug_enabled is True but debug_output_dir is None
    ///         - ConfigurationError if zerobus_writer_disabled is True but debug_enabled is False
    #[new]
    #[pyo3(signature = (endpoint, table_name, *, client_id=None, client_secret=None, unity_catalog_url=None, observability_enabled=false, observability_config=None, debug_enabled=false, debug_output_dir=None, debug_flush_interval_secs=5, debug_max_file_size=None, retry_max_attempts=5, retry_base_delay_ms=100, retry_max_delay_ms=30000, zerobus_writer_disabled=false))]
    pub fn new(
        endpoint: String,
        table_name: String,
        client_id: Option<String>,
        client_secret: Option<String>,
        unity_catalog_url: Option<String>,
        observability_enabled: bool,
        observability_config: Option<PyObject>,
        debug_enabled: bool,
        debug_output_dir: Option<String>,
        debug_flush_interval_secs: u64,
        debug_max_file_size: Option<u64>,
        retry_max_attempts: u32,
        retry_base_delay_ms: u64,
        retry_max_delay_ms: u64,
        zerobus_writer_disabled: bool,
    ) -> PyResult<Self> {
        let mut config = WrapperConfiguration::new(endpoint, table_name);

        if let (Some(cid), Some(cs)) = (client_id, client_secret) {
            config = config.with_credentials(cid, cs);
        }

        if let Some(url) = unity_catalog_url {
            config = config.with_unity_catalog(url);
        }

        if observability_enabled {
            let otlp_config = if let Some(config_obj) = observability_config {
                Python::with_gil(|py| {
                    let dict = config_obj.extract::<&PyDict>(py)?;
                    let endpoint = dict
                        .get_item("endpoint")?
                        .and_then(|v| v.extract::<String>().ok());

                    let output_dir = dict
                        .get_item("output_dir")?
                        .and_then(|v| v.extract::<String>().ok())
                        .map(std::path::PathBuf::from);

                    let write_interval_secs = dict
                        .get_item("write_interval_secs")?
                        .and_then(|v| v.extract::<u64>().ok())
                        .unwrap_or(5);

                    let log_level = dict
                        .get_item("log_level")?
                        .and_then(|v| v.extract::<String>().ok())
                        .unwrap_or_else(|| "info".to_string());

                    let otlp_config = OtlpSdkConfig {
                        endpoint,
                        output_dir,
                        write_interval_secs,
                        log_level,
                    };
                    // Validate configuration before using it
                    otlp_config.validate().map_err(|e| {
                        PyException::new_err(format!("Invalid OTLP SDK configuration: {}", e))
                    })?;
                    Ok::<OtlpSdkConfig, PyErr>(otlp_config)
                })?
            } else {
                OtlpSdkConfig::default()
            };
            config = config.with_observability(otlp_config);
        }

        if debug_enabled {
            if let Some(output_dir) = debug_output_dir {
                config = config.with_debug_output(PathBuf::from(output_dir));
                config.debug_flush_interval_secs = debug_flush_interval_secs;
                config.debug_max_file_size = debug_max_file_size;
            } else {
                // If debug_enabled is True but debug_output_dir is None, raise an error
                // This prevents silent failure where debug_enabled is ignored
                return Err(PyConfigurationError::new_err(
                    "debug_output_dir is required when debug_enabled is True. \
                    Either provide debug_output_dir or set debug_enabled=False."
                        .to_string(),
                ));
            }
        }

        config =
            config.with_retry_config(retry_max_attempts, retry_base_delay_ms, retry_max_delay_ms);

        if zerobus_writer_disabled {
            config = config.with_zerobus_writer_disabled(true);
        }

        Ok(Self { inner: config })
    }

    fn validate(&self) -> PyResult<()> {
        self.inner.validate().map_err(rust_error_to_python_error)?;
        Ok(())
    }

    // Getters for configuration fields
    #[getter]
    fn endpoint(&self) -> String {
        self.inner.zerobus_endpoint.clone()
    }

    #[getter]
    fn table_name(&self) -> String {
        self.inner.table_name.clone()
    }

    #[getter]
    fn client_id(&self) -> Option<String> {
        use secrecy::ExposeSecret;
        self.inner
            .client_id
            .as_ref()
            .map(|s| s.expose_secret().to_string())
    }

    #[getter]
    fn client_secret(&self) -> Option<String> {
        use secrecy::ExposeSecret;
        self.inner
            .client_secret
            .as_ref()
            .map(|s| s.expose_secret().to_string())
    }

    #[getter]
    fn unity_catalog_url(&self) -> Option<String> {
        self.inner.unity_catalog_url.clone()
    }

    #[getter]
    fn debug_enabled(&self) -> bool {
        self.inner.debug_enabled
    }

    #[getter]
    fn debug_output_dir(&self) -> Option<String> {
        self.inner
            .debug_output_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
    }

    #[getter]
    fn debug_flush_interval_secs(&self) -> u64 {
        self.inner.debug_flush_interval_secs
    }

    #[getter]
    fn debug_max_file_size(&self) -> Option<u64> {
        self.inner.debug_max_file_size
    }

    #[getter]
    fn retry_max_attempts(&self) -> u32 {
        self.inner.retry_max_attempts
    }

    #[getter]
    fn retry_base_delay_ms(&self) -> u64 {
        self.inner.retry_base_delay_ms
    }

    #[getter]
    fn retry_max_delay_ms(&self) -> u64 {
        self.inner.retry_max_delay_ms
    }

    #[getter]
    fn observability_enabled(&self) -> bool {
        self.inner.observability_enabled
    }

    #[getter]
    fn zerobus_writer_disabled(&self) -> bool {
        self.inner.zerobus_writer_disabled
    }
}

/// Python wrapper for TransmissionResult
#[pyclass(name = "TransmissionResult")]
#[derive(Clone)]
pub struct PyTransmissionResult {
    // Made pub for tests (which are in a separate crate)
    #[allow(dead_code)] // Used in tests
    pub inner: TransmissionResult,
}

#[pymethods]
impl PyTransmissionResult {
    #[getter]
    pub fn success(&self) -> bool {
        self.inner.success
    }

    #[getter]
    pub fn error(&self) -> Option<String> {
        self.inner.error.as_ref().map(|e| e.to_string())
    }

    #[getter]
    pub fn attempts(&self) -> u32 {
        self.inner.attempts
    }

    #[getter]
    pub fn latency_ms(&self) -> Option<u64> {
        self.inner.latency_ms
    }

    #[getter]
    pub fn batch_size_bytes(&self) -> usize {
        self.inner.batch_size_bytes
    }

    /// Get failed rows with their errors
    ///
    /// Returns a list of tuples (row_index, error_message) for rows that failed.
    /// Returns None if all rows succeeded.
    #[getter]
    pub fn failed_rows(&self) -> Option<Vec<(usize, String)>> {
        self.inner.failed_rows.as_ref().map(|rows| {
            rows.iter()
                .map(|(idx, error)| (*idx, error.to_string()))
                .collect()
        })
    }

    /// Get indices of successfully written rows
    ///
    /// Returns a list of row indices that were successfully written.
    /// Returns None if all rows failed.
    #[getter]
    pub fn successful_rows(&self) -> Option<Vec<usize>> {
        self.inner.successful_rows.clone()
    }

    /// Get total number of rows in the batch
    #[getter]
    pub fn total_rows(&self) -> usize {
        self.inner.total_rows
    }

    /// Get count of successfully written rows
    #[getter]
    pub fn successful_count(&self) -> usize {
        self.inner.successful_count
    }

    /// Get count of failed rows
    #[getter]
    pub fn failed_count(&self) -> usize {
        self.inner.failed_count
    }

    /// Get indices of failed rows
    ///
    /// Returns a list of row indices that failed, or empty list if none failed.
    pub fn get_failed_row_indices(&self) -> Vec<usize> {
        self.inner.get_failed_row_indices()
    }

    /// Get indices of successful rows
    ///
    /// Returns a list of row indices that succeeded, or empty list if none succeeded.
    pub fn get_successful_row_indices(&self) -> Vec<usize> {
        self.inner.get_successful_row_indices()
    }

    /// Extract a RecordBatch containing only the failed rows from the original batch
    ///
    /// Args:
    ///     original_batch: The original PyArrow RecordBatch that was sent
    ///
    /// Returns:
    ///     PyArrow RecordBatch containing only the failed rows, or None if there are no failed rows.
    ///     Rows are extracted in the order they appear in failed_rows.
    pub fn extract_failed_batch(
        &self,
        py: Python,
        original_batch: PyObject,
    ) -> PyResult<Option<PyObject>> {
        let rust_batch = pyarrow_to_rust_batch(py, original_batch)?;

        match self.inner.extract_failed_batch(&rust_batch) {
            Some(batch) => {
                // Convert Rust RecordBatch back to PyArrow RecordBatch
                let py_batch = rust_batch_to_pyarrow(py, &batch)?;
                Ok(Some(py_batch))
            }
            None => Ok(None),
        }
    }

    /// Extract a RecordBatch containing only the successful rows from the original batch
    ///
    /// Args:
    ///     original_batch: The original PyArrow RecordBatch that was sent
    ///
    /// Returns:
    ///     PyArrow RecordBatch containing only the successful rows, or None if there are no successful rows.
    ///     Rows are extracted in the order they appear in successful_rows.
    pub fn extract_successful_batch(
        &self,
        py: Python,
        original_batch: PyObject,
    ) -> PyResult<Option<PyObject>> {
        let rust_batch = pyarrow_to_rust_batch(py, original_batch)?;

        match self.inner.extract_successful_batch(&rust_batch) {
            Some(batch) => {
                // Convert Rust RecordBatch back to PyArrow RecordBatch
                let py_batch = rust_batch_to_pyarrow(py, &batch)?;
                Ok(Some(py_batch))
            }
            None => Ok(None),
        }
    }

    /// Get indices of failed rows filtered by error type
    ///
    /// Args:
    ///     error_type: String representing the error type to filter by
    ///                 (e.g., "ConversionError", "TransmissionError", "ConnectionError")
    ///
    /// Returns:
    ///     List of row indices for failed rows that match the error type.
    pub fn get_failed_row_indices_by_error_type(&self, error_type: &str) -> Vec<usize> {
        self.inner
            .get_failed_row_indices_by_error_type(|error| match error_type {
                "ConversionError" => matches!(error, ZerobusError::ConversionError(_)),
                "TransmissionError" => matches!(error, ZerobusError::TransmissionError(_)),
                "ConnectionError" => matches!(error, ZerobusError::ConnectionError(_)),
                "AuthenticationError" => matches!(error, ZerobusError::AuthenticationError(_)),
                "ConfigurationError" => matches!(error, ZerobusError::ConfigurationError(_)),
                "RetryExhausted" => matches!(error, ZerobusError::RetryExhausted(_)),
                "TokenRefreshError" => matches!(error, ZerobusError::TokenRefreshError(_)),
                _ => false,
            })
    }

    /// Check if this result represents a partial success (some rows succeeded, some failed)
    ///
    /// Returns:
    ///     True if there are both successful and failed rows.
    pub fn is_partial_success(&self) -> bool {
        self.inner.is_partial_success()
    }

    /// Check if there are any failed rows
    ///
    /// Returns:
    ///     True if failed_rows contains any entries.
    pub fn has_failed_rows(&self) -> bool {
        self.inner.has_failed_rows()
    }

    /// Check if there are any successful rows
    ///
    /// Returns:
    ///     True if successful_rows contains any entries.
    pub fn has_successful_rows(&self) -> bool {
        self.inner.has_successful_rows()
    }

    /// Group failed rows by error type
    ///
    /// Returns:
    ///     Dictionary mapping error type names to lists of row indices.
    pub fn group_errors_by_type(&self) -> HashMap<String, Vec<usize>> {
        self.inner.group_errors_by_type()
    }

    /// Get error statistics for this transmission result
    ///
    /// Returns:
    ///     Dictionary containing error statistics including:
    ///     - total_rows: Total number of rows
    ///     - successful_count: Number of successful rows
    ///     - failed_count: Number of failed rows
    ///     - success_rate: Success rate (0.0 to 1.0)
    ///     - failure_rate: Failure rate (0.0 to 1.0)
    ///     - error_type_counts: Dictionary mapping error types to counts
    pub fn get_error_statistics(&self, py: Python) -> PyResult<PyObject> {
        let stats = self.inner.get_error_statistics();
        let dict = PyDict::new(py);
        dict.set_item("total_rows", stats.total_rows)?;
        dict.set_item("successful_count", stats.successful_count)?;
        dict.set_item("failed_count", stats.failed_count)?;
        dict.set_item("success_rate", stats.success_rate)?;
        dict.set_item("failure_rate", stats.failure_rate)?;

        let error_type_counts = PyDict::new(py);
        for (error_type, count) in stats.error_type_counts {
            error_type_counts.set_item(error_type, count)?;
        }
        dict.set_item("error_type_counts", error_type_counts)?;

        Ok(dict.to_object(py))
    }

    /// Get all error messages from failed rows
    ///
    /// Returns:
    ///     List of error message strings for all failed rows.
    pub fn get_error_messages(&self) -> Vec<String> {
        self.inner.get_error_messages()
    }
}

/// Python wrapper for ZerobusWrapper
///
/// Thread-safe wrapper that handles Arrow RecordBatch to Protobuf conversion,
/// authentication, retry logic, and transmission to Zerobus.
#[pyclass(name = "ZerobusWrapper")]
#[allow(non_local_definitions)]
pub struct PyZerobusWrapper {
    inner: Arc<ZerobusWrapper>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyZerobusWrapper {
    #[new]
    fn new(config: PyWrapperConfiguration) -> PyResult<Self> {
        // Validate configuration
        config.validate()?;

        // Create Tokio runtime for async operations
        let runtime = Runtime::new()
            .map_err(|e| PyException::new_err(format!("Failed to create Tokio runtime: {}", e)))?;

        // Initialize wrapper
        let wrapper = runtime.block_on(async {
            ZerobusWrapper::new(config.inner.clone())
                .await
                .map_err(rust_error_to_python_error)
        })?;

        Ok(Self {
            inner: Arc::new(wrapper),
            runtime: Arc::new(runtime),
        })
    }

    /// Send an Arrow RecordBatch to Zerobus.
    ///
    /// Converts PyArrow RecordBatch to Rust RecordBatch and transmits to Zerobus
    /// with automatic retry on transient failures.
    ///
    /// Args:
    ///     batch: PyArrow RecordBatch to send
    ///
    /// Returns:
    ///     TransmissionResult indicating success or failure
    ///
    /// Raises:
    ///     ZerobusError: If transmission fails after all retry attempts
    fn send_batch(&self, py: Python, batch: PyObject) -> PyResult<PyTransmissionResult> {
        // Convert PyArrow RecordBatch to Rust RecordBatch
        // This uses zero-copy conversion via PyArrow's C data interface
        let rust_batch = pyarrow_to_rust_batch(py, batch)?;

        // Execute async operation on Tokio runtime
        let result = self
            .runtime
            .block_on(async { self.inner.send_batch(rust_batch).await });

        match result {
            Ok(transmission_result) => Ok(PyTransmissionResult {
                inner: transmission_result,
            }),
            Err(e) => Err(rust_error_to_python_error(e)),
        }
    }

    /// Flush any pending operations and ensure data is transmitted.
    ///
    /// Raises:
    ///     ZerobusError: If flush operation fails
    fn flush(&self, _py: Python) -> PyResult<()> {
        self.runtime
            .block_on(async { self.inner.flush().await })
            .map_err(rust_error_to_python_error)?;
        Ok(())
    }

    /// Shutdown the wrapper gracefully, closing connections and cleaning up resources.
    ///
    /// Raises:
    ///     ZerobusError: If shutdown fails
    fn shutdown(&self, _py: Python) -> PyResult<()> {
        self.runtime
            .block_on(async { self.inner.shutdown().await })
            .map_err(rust_error_to_python_error)?;
        Ok(())
    }

    /// Async context manager entry
    fn __aenter__(&self) -> PyResult<Self> {
        Ok(self.clone())
    }

    /// Async context manager exit
    fn __aexit__(
        &self,
        _py: Python,
        _exc_type: PyObject,
        _exc_val: PyObject,
        _exc_tb: PyObject,
    ) -> PyResult<()> {
        self.shutdown(_py)?;
        Ok(())
    }
}

impl Clone for PyZerobusWrapper {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            runtime: Arc::clone(&self.runtime),
        }
    }
}

/// Convert PyArrow RecordBatch to Rust RecordBatch
///
/// Uses PyArrow's C data interface for efficient conversion when possible.
/// Falls back to Python API extraction if C data interface is not available.
fn pyarrow_to_rust_batch(py: Python, batch: PyObject) -> PyResult<RecordBatch> {
    // Import PyArrow module
    let pyarrow = PyModule::import(py, "pyarrow")?;

    // Get RecordBatch class
    let record_batch_class = pyarrow.getattr("RecordBatch")?;

    // Check if the object is a RecordBatch
    let batch_ref = batch.as_ref(py);
    if !batch_ref.is_instance(record_batch_class)? {
        return Err(PyTypeError::new_err(
            "Expected pyarrow.RecordBatch, got different type",
        ));
    }

    // Try to use PyArrow's C data interface for zero-copy conversion
    // This is the most efficient method
    if let Ok(c_batch) = pyarrow_to_rust_batch_c_interface(py, batch_ref) {
        return Ok(c_batch);
    }

    // Fallback: Use PyArrow's Python API to extract data
    // This is less efficient but works for all PyArrow versions
    pyarrow_to_rust_batch_python_api(py, batch_ref)
}

/// Convert PyArrow RecordBatch using C data interface (zero-copy when possible)
///
/// Uses PyArrow's IPC serialization as an efficient intermediate format.
/// PyArrow's `to_pybytes()` serializes to Arrow IPC format, which can be
/// efficiently deserialized in Rust without copying individual array elements.
fn pyarrow_to_rust_batch_c_interface(_py: Python, batch_ref: &PyAny) -> PyResult<RecordBatch> {
    use arrow::ipc::reader::StreamReader;
    use std::io::Cursor;

    // Use PyArrow's IPC serialization for efficient conversion
    // This avoids copying individual array elements by using Arrow's
    // binary format as an intermediate representation

    // Serialize RecordBatch to IPC format using PyArrow
    let serialized = batch_ref.call_method0("to_pybytes")?;
    let bytes: Vec<u8> = serialized.extract()?;

    // Deserialize in Rust using Arrow IPC reader
    // This is efficient because Arrow IPC format matches Rust Arrow format
    let cursor = Cursor::new(bytes);
    let mut reader = StreamReader::try_new(cursor, None)
        .map_err(|e| PyException::new_err(format!("Failed to create IPC reader: {}", e)))?;

    // Read the RecordBatch from the IPC stream
    let batch = reader
        .next()
        .ok_or_else(|| PyException::new_err("No RecordBatch in IPC stream"))?
        .map_err(|e| PyException::new_err(format!("Failed to read RecordBatch: {}", e)))?;

    Ok(batch)
}

/// Convert PyArrow RecordBatch using Python API (fallback method)
fn pyarrow_to_rust_batch_python_api(py: Python, batch_ref: &PyAny) -> PyResult<RecordBatch> {
    use arrow::array::*;
    use arrow::datatypes::{Field, Schema};
    use std::sync::Arc;

    // Get schema from PyArrow RecordBatch
    // PyArrow Schema objects are sequences - they support len() and indexing
    let schema_obj = batch_ref.getattr("schema")?;

    // Get number of fields using len() method (Schema objects support len())
    let num_fields = schema_obj.call_method0("__len__")?.extract::<usize>()?;

    let mut rust_fields = Vec::new();
    let mut rust_arrays = Vec::new();

    // Convert each field and array
    // PyArrow Schema objects support indexing: schema[i] returns the field
    for i in 0..num_fields {
        let field_obj = schema_obj.get_item(i)?;
        let field_name = field_obj.getattr("name")?.extract::<String>()?;
        let field_type_obj = field_obj.getattr("type")?;
        let field_type_str = format!("{}", field_type_obj);

        // Map PyArrow type to Rust Arrow type
        let rust_type = pyarrow_type_to_rust_type(&field_type_str)?;
        rust_fields.push(Field::new(field_name.clone(), rust_type.clone(), true));

        // Get array from batch
        let array_obj = batch_ref.call_method1("column", (i,))?;

        // Convert PyArrow array to Rust array
        let rust_array = pyarrow_array_to_rust_array(py, array_obj, &rust_type)?;
        rust_arrays.push(rust_array);
    }

    // Create Rust RecordBatch
    let schema = Schema::new(rust_fields);
    RecordBatch::try_new(Arc::new(schema), rust_arrays)
        .map_err(|e| PyException::new_err(format!("Failed to create RecordBatch: {}", e)))
}

/// Convert PyArrow type string to Rust Arrow DataType
fn pyarrow_type_to_rust_type(type_str: &str) -> PyResult<DataType> {
    // Map PyArrow type strings to Rust Arrow types
    // This is a simplified mapping - full implementation should handle all types
    if type_str.contains("int64") {
        Ok(DataType::Int64)
    } else if type_str.contains("int32") {
        Ok(DataType::Int32)
    } else if type_str.contains("string") || type_str.contains("utf8") {
        Ok(DataType::Utf8)
    } else if type_str.contains("float64") || type_str.contains("double") {
        Ok(DataType::Float64)
    } else if type_str.contains("float32") || type_str.contains("float") {
        Ok(DataType::Float32)
    } else if type_str.contains("bool") {
        Ok(DataType::Boolean)
    } else if type_str.contains("binary") {
        Ok(DataType::Binary)
    } else {
        Err(PyNotImplementedError::new_err(format!(
            "Unsupported PyArrow type: {}",
            type_str
        )))
    }
}

/// Convert PyArrow array to Rust Arrow array
fn pyarrow_array_to_rust_array(
    _py: Python,
    array_obj: &PyAny,
    data_type: &DataType,
) -> PyResult<Arc<dyn arrow::array::Array>> {
    use arrow::array::*;
    use std::sync::Arc;

    // Get array length
    // PyArrow arrays support __len__() method, not a len attribute
    let len = array_obj.call_method0("__len__")?.extract::<usize>()?;

    match data_type {
        DataType::Int64 => {
            let values: Vec<Option<i64>> = (0..len)
                .map(|i| {
                    let val = array_obj.get_item(i)?;
                    if val.is_none() {
                        Ok(None)
                    } else {
                        Ok(Some(val.extract::<i64>()?))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Arc::new(Int64Array::from(values)))
        }
        DataType::Utf8 => {
            let values: Vec<Option<String>> = (0..len)
                .map(|i| {
                    let val = array_obj.get_item(i)?;
                    if val.is_none() {
                        Ok(None)
                    } else {
                        // PyArrow returns StringScalar objects, not plain strings
                        // Use as_py() method to convert to Python string, then extract
                        let py_str = if val.hasattr("as_py")? {
                            val.call_method0("as_py")?
                        } else {
                            // Fallback: convert to string representation
                            val.call_method0("__str__")?
                        };
                        Ok(Some(py_str.extract::<String>()?))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Arc::new(StringArray::from(values)))
        }
        DataType::Float64 => {
            let values: Vec<Option<f64>> = (0..len)
                .map(|i| {
                    let val = array_obj.get_item(i)?;
                    if val.is_none() {
                        Ok(None)
                    } else {
                        Ok(Some(val.extract::<f64>()?))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Arc::new(Float64Array::from(values)))
        }
        DataType::Boolean => {
            let values: Vec<Option<bool>> = (0..len)
                .map(|i| {
                    let val = array_obj.get_item(i)?;
                    if val.is_none() {
                        Ok(None)
                    } else {
                        Ok(Some(val.extract::<bool>()?))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Arc::new(BooleanArray::from(values)))
        }
        _ => Err(PyNotImplementedError::new_err(format!(
            "Array type conversion not yet implemented for: {:?}",
            data_type
        ))),
    }
}

/// Convert Rust RecordBatch to PyArrow RecordBatch
///
/// Uses Arrow IPC serialization as an efficient intermediate format.
/// Serializes the Rust RecordBatch to IPC format, then deserializes it in Python.
fn rust_batch_to_pyarrow(py: Python, batch: &RecordBatch) -> PyResult<PyObject> {
    use arrow::ipc::writer::StreamWriter;
    use pyo3::types::PyBytes;
    use std::io::Cursor;

    // Serialize Rust RecordBatch to IPC format
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = StreamWriter::try_new(cursor, &batch.schema())
        .map_err(|e| PyException::new_err(format!("Failed to create IPC writer: {}", e)))?;

    writer
        .write(batch)
        .map_err(|e| PyException::new_err(format!("Failed to write RecordBatch: {}", e)))?;

    writer
        .finish()
        .map_err(|e| PyException::new_err(format!("Failed to finish IPC writer: {}", e)))?;

    // Import PyArrow module
    let pyarrow = PyModule::import(py, "pyarrow")?;

    // Get RecordBatch class
    let record_batch_class = pyarrow.getattr("RecordBatch")?;

    // Deserialize IPC bytes in Python using PyArrow
    let ipc_bytes = PyBytes::new(py, &buffer);
    let py_batch = record_batch_class.call_method1("from_pybytes", (ipc_bytes,))?;

    Ok(py_batch.to_object(py))
}

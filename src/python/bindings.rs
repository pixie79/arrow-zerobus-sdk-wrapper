//! PyO3 bindings implementation
//!
//! This module implements Python bindings for the Zerobus SDK Wrapper,
//! providing a Pythonic API that matches the Rust API functionality.

// PyO3's #[pymethods] macro generates non-local impl blocks, which is necessary for bindings
// This lint must be disabled for PyO3 bindings to work correctly
#![allow(non_local_definitions)]

use crate::config::OtlpConfig;
use crate::config::WrapperConfiguration;
use crate::error::ZerobusError;
use crate::wrapper::{TransmissionResult, ZerobusWrapper};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use pyo3::exceptions::{PyException, PyNotImplementedError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
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
pub(crate) fn rust_error_to_python_error(error: ZerobusError) -> PyErr {
    match error {
        ZerobusError::ConfigurationError(msg) => PyConfigurationError::new_err(msg),
        ZerobusError::AuthenticationError(msg) => PyAuthenticationError::new_err(msg),
        ZerobusError::ConnectionError(msg) => PyConnectionError::new_err(msg),
        ZerobusError::ConversionError(msg) => PyConversionError::new_err(msg),
        ZerobusError::TransmissionError(msg) => PyTransmissionError::new_err(msg),
        ZerobusError::RetryExhausted(msg) => PyRetryExhausted::new_err(msg),
        ZerobusError::TokenRefreshError(msg) => PyTokenRefreshError::new_err(msg),
    }
}

// Exception classes
// Note: In PyO3, all custom exceptions must extend PyException directly.
// We cannot use a custom base class (PyZerobusError) for other exceptions
// because PyO3 doesn't support that pattern. Instead, all exceptions extend
// PyException directly, but they're logically grouped as ZerobusError exceptions.
#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyZerobusError;

#[pymethods]
impl PyZerobusError {
    // Base exception class for Zerobus errors
}

#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyConfigurationError;

#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyAuthenticationError;

#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyConnectionError;

#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyConversionError;

#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyTransmissionError;

#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyRetryExhausted;

#[pyclass(extends=PyException)]
#[derive(Debug)]
pub struct PyTokenRefreshError;

impl PyConfigurationError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyConfigurationError, _>(msg)
    }
}

impl PyAuthenticationError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyAuthenticationError, _>(msg)
    }
}

impl PyConnectionError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyConnectionError, _>(msg)
    }
}

impl PyConversionError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyConversionError, _>(msg)
    }
}

impl PyTransmissionError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyTransmissionError, _>(msg)
    }
}

impl PyRetryExhausted {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyRetryExhausted, _>(msg)
    }
}

impl PyTokenRefreshError {
    fn new_err(msg: String) -> PyErr {
        PyErr::new::<PyTokenRefreshError, _>(msg)
    }
}

/// Python wrapper for WrapperConfiguration
#[pyclass]
#[derive(Clone)]
#[allow(non_local_definitions)]
pub struct PyWrapperConfiguration {
    inner: WrapperConfiguration,
}

#[pymethods]
#[allow(clippy::too_many_arguments)]
impl PyWrapperConfiguration {
    #[new]
    #[pyo3(signature = (endpoint, table_name, *, client_id=None, client_secret=None, unity_catalog_url=None, observability_enabled=false, observability_config=None, debug_enabled=false, debug_output_dir=None, debug_flush_interval_secs=5, debug_max_file_size=None, retry_max_attempts=5, retry_base_delay_ms=100, retry_max_delay_ms=30000))]
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

                    // Extract any additional configuration from the dict
                    let mut extra = std::collections::HashMap::new();
                    for (key, value) in dict.iter() {
                        let key_str = key.extract::<String>()?;
                        if key_str != "endpoint" {
                            // Try to extract as JSON-serializable value
                            if let Ok(json_str) = value.extract::<String>() {
                                if let Ok(json_val) =
                                    serde_json::from_str::<serde_json::Value>(&json_str)
                                {
                                    extra.insert(key_str, json_val);
                                }
                            }
                        }
                    }

                    Ok::<OtlpConfig, PyErr>(OtlpConfig { endpoint, extra })
                })?
            } else {
                OtlpConfig::default()
            };
            config = config.with_observability(otlp_config);
        }

        if debug_enabled {
            if let Some(output_dir) = debug_output_dir {
                config = config.with_debug_output(PathBuf::from(output_dir));
                config.debug_flush_interval_secs = debug_flush_interval_secs;
                config.debug_max_file_size = debug_max_file_size;
            }
        }

        config =
            config.with_retry_config(retry_max_attempts, retry_base_delay_ms, retry_max_delay_ms);

        Ok(Self { inner: config })
    }

    fn validate(&self) -> PyResult<()> {
        self.inner.validate().map_err(rust_error_to_python_error)?;
        Ok(())
    }
}

/// Python wrapper for TransmissionResult
#[pyclass]
#[derive(Clone)]
pub struct PyTransmissionResult {
    #[allow(dead_code)] // Used in tests
    pub(crate) inner: TransmissionResult,
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
}

/// Python wrapper for ZerobusWrapper
///
/// Thread-safe wrapper that handles Arrow RecordBatch to Protobuf conversion,
/// authentication, retry logic, and transmission to Zerobus.
#[pyclass]
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
    let schema_obj = batch_ref.getattr("schema")?;
    let schema_fields = schema_obj.getattr("fields")?;
    let num_fields = schema_fields.len()?;

    let mut rust_fields = Vec::new();
    let mut rust_arrays = Vec::new();

    // Convert each field and array
    for i in 0..num_fields {
        let field_obj = schema_fields.get_item(i)?;
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
    let len = array_obj.getattr("len")?.extract::<usize>()?;

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
                        Ok(Some(val.extract::<String>()?))
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

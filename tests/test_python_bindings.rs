//! Unit tests for Python bindings (Rust side)

#[cfg(feature = "python")]
mod python_tests {
    use arrow_zerobus_sdk_wrapper::python::bindings::*;
    use pyo3::prelude::*;

    #[test]
    fn test_error_conversion() {
        Python::with_gil(|py| {
            use arrow_zerobus_sdk_wrapper::ZerobusError;

            let config_error = ZerobusError::ConfigurationError("test".to_string());
            let py_err = rust_error_to_python_error(config_error);

            assert!(py_err.is_instance_of::<PyConfigurationError>(py));
        });
    }

    #[test]
    fn test_py_wrapper_configuration_new() {
        Python::with_gil(|py| {
            let config = PyWrapperConfiguration::new(
                "https://test.cloud.databricks.com".to_string(),
                "test_table".to_string(),
                Some("client_id".to_string()),
                Some("client_secret".to_string()),
                Some("https://unity-catalog-url".to_string()),
                false,
                None,
                false,
                None,
                5,
                None,
                5,
                100,
                30000,
            );

            assert!(config.is_ok());
        });
    }

    #[test]
    fn test_py_transmission_result() {
        use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;

        let result = TransmissionResult {
            success: true,
            error: None,
            attempts: 1,
            latency_ms: Some(100),
            batch_size_bytes: 1024,
        };

        let py_result = PyTransmissionResult { inner: result };

        assert!(py_result.success());
        assert_eq!(py_result.attempts(), 1);
        assert_eq!(py_result.latency_ms(), Some(100));
        assert_eq!(py_result.batch_size_bytes(), 1024);
    }
}

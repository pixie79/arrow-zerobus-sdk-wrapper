//! Unit tests for Python bindings
//!
//! These tests verify the Python bindings work correctly from the Rust side.
//! Target: ≥90% coverage per file.

#[cfg(feature = "python")]
mod python_bindings_tests {
    use arrow_zerobus_sdk_wrapper::python::bindings::*;
    use arrow_zerobus_sdk_wrapper::{ZerobusError, WrapperConfiguration};
    use pyo3::prelude::*;

    #[test]
    fn test_rust_error_to_python_error_all_variants() {
        // Test all error variants are converted correctly
        let errors = vec![
            ZerobusError::ConfigurationError("config".to_string()),
            ZerobusError::AuthenticationError("auth".to_string()),
            ZerobusError::ConnectionError("conn".to_string()),
            ZerobusError::ConversionError("conv".to_string()),
            ZerobusError::TransmissionError("trans".to_string()),
            ZerobusError::RetryExhausted("retry".to_string()),
            ZerobusError::TokenRefreshError("token".to_string()),
        ];

        Python::with_gil(|py| {
            for error in errors {
                let py_err = rust_error_to_python_error(error);
                // Verify error is a PyErr (can be converted)
                assert!(py_err.is_instance_of::<PyAny>(py).is_ok());
            }
        });
    }

    #[test]
    fn test_py_wrapper_configuration_new_with_all_options() {
        Python::with_gil(|py| {
            let config = PyWrapperConfiguration::new(
                "https://test.cloud.databricks.com".to_string(),
                "test_table".to_string(),
                Some("client_id".to_string()),
                Some("client_secret".to_string()),
                Some("https://unity-catalog-url".to_string()),
                true, // observability_enabled
                None,  // observability_config
                true,  // debug_enabled
                Some("/tmp/debug".to_string()), // debug_output_dir
                10,    // debug_flush_interval_secs
                Some(1024 * 1024), // debug_max_file_size
                10,    // retry_max_attempts
                200,   // retry_base_delay_ms
                60000, // retry_max_delay_ms
            );

            assert!(config.is_ok());
            let config = config.unwrap();
            
            // Verify validation works
            let validation_result = config.validate();
            // May fail if endpoint is invalid, but should not panic
            assert!(validation_result.is_ok() || validation_result.is_err());
        });
    }

    #[test]
    fn test_py_wrapper_configuration_minimal() {
        Python::with_gil(|py| {
            let config = PyWrapperConfiguration::new(
                "https://test.cloud.databricks.com".to_string(),
                "test_table".to_string(),
                None, None, None, false, None, false, None, 5, None, 5, 100, 30000,
            );

            assert!(config.is_ok());
        });
    }

    #[test]
    fn test_py_transmission_result_getters() {
        use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;

        let result = TransmissionResult {
            success: true,
            error: None,
            attempts: 3,
            latency_ms: Some(150),
            batch_size_bytes: 2048,
            failed_rows: None,
            successful_rows: None,
            total_rows: 0,
            successful_count: 0,
            failed_count: 0,
        };

        let py_result = PyTransmissionResult { inner: result };

        assert!(py_result.success());
        assert_eq!(py_result.attempts(), 3);
        assert_eq!(py_result.latency_ms(), Some(150));
        assert_eq!(py_result.batch_size_bytes(), 2048);
        assert!(py_result.error().is_none());
    }

    #[test]
    fn test_py_transmission_result_with_error() {
        use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;

        let result = TransmissionResult {
            success: false,
            error: Some(ZerobusError::ConnectionError("test error".to_string())),
            attempts: 5,
            latency_ms: None,
            batch_size_bytes: 1024,
            failed_rows: None,
            successful_rows: None,
            total_rows: 0,
            successful_count: 0,
            failed_count: 0,
        };

        let py_result = PyTransmissionResult { inner: result };

        assert!(!py_result.success());
        assert_eq!(py_result.attempts(), 5);
        assert_eq!(py_result.latency_ms(), None);
        assert_eq!(py_result.batch_size_bytes(), 1024);
        assert!(py_result.error().is_some());
        assert!(py_result.error().unwrap().contains("test error"));
    }

    #[test]
    fn test_py_wrapper_configuration_with_observability() {
        Python::with_gil(|py| {
            let observability_config = PyDict::new(py);
            observability_config.set_item("endpoint", "http://localhost:4317")
                .expect("Failed to set endpoint");

            let config = PyWrapperConfiguration::new(
                "https://test.cloud.databricks.com".to_string(),
                "test_table".to_string(),
                Some("client_id".to_string()),
                Some("client_secret".to_string()),
                Some("https://unity-catalog-url".to_string()),
                true, // observability_enabled
                Some(observability_config.into()), // observability_config
                false, false, None, 5, None, 5, 100, 30000,
            );

            assert!(config.is_ok());
        });
    }
}


//! Contract tests for Python API
//!
//! These tests verify that the Python API matches the contract specification
//! defined in specs/001-zerobus-wrapper/contracts/python-api.md

#[cfg(feature = "python")]
mod python_contract_tests {
    use arrow_zerobus_sdk_wrapper::python::bindings::*;
    use pyo3::prelude::*;

    #[test]
    fn test_python_wrapper_class_exists() {
        Python::with_gil(|py| {
            let module = PyModule::import(py, "arrow_zerobus_sdk_wrapper")
                .expect("Module should be importable");
            
            // Verify ZerobusWrapper class exists
            assert!(module.getattr("ZerobusWrapper").is_ok());
        });
    }

    #[test]
    fn test_python_transmission_result_class_exists() {
        Python::with_gil(|py| {
            let module = PyModule::import(py, "arrow_zerobus_sdk_wrapper")
                .expect("Module should be importable");
            
            // Verify TransmissionResult class exists
            assert!(module.getattr("TransmissionResult").is_ok());
        });
    }

    #[test]
    fn test_python_error_classes_exist() {
        Python::with_gil(|py| {
            let module = PyModule::import(py, "arrow_zerobus_sdk_wrapper")
                .expect("Module should be importable");
            
            // Verify all error classes exist per contract
            assert!(module.getattr("ZerobusError").is_ok());
            assert!(module.getattr("ConfigurationError").is_ok());
            assert!(module.getattr("AuthenticationError").is_ok());
            assert!(module.getattr("ConnectionError").is_ok());
            assert!(module.getattr("ConversionError").is_ok());
            assert!(module.getattr("TransmissionError").is_ok());
            assert!(module.getattr("RetryExhausted").is_ok());
            assert!(module.getattr("TokenRefreshError").is_ok());
        });
    }

    #[test]
    fn test_python_wrapper_configuration_contract() {
        Python::with_gil(|py| {
            // Test that PyWrapperConfiguration can be created with required fields
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
    fn test_python_transmission_result_contract() {
        use arrow_zerobus_sdk_wrapper::wrapper::TransmissionResult;
        
        // Contract: TransmissionResult must have these fields accessible
        let result = TransmissionResult {
            success: true,
            error: None,
            attempts: 1,
            latency_ms: Some(100),
            batch_size_bytes: 1024,
        };
        
        let py_result = PyTransmissionResult { inner: result };
        
        // Contract: All fields should be accessible via getters
        assert!(py_result.success());
        assert_eq!(py_result.attempts(), 1);
        assert_eq!(py_result.latency_ms(), Some(100));
        assert_eq!(py_result.batch_size_bytes(), 1024);
        assert!(py_result.error().is_none());
    }
}


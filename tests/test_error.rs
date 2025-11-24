//! Integration tests for error types

use arrow_zerobus_sdk_wrapper::ZerobusError;

#[test]
fn test_error_is_retryable() {
    let connection_error = ZerobusError::ConnectionError("test".to_string());
    assert!(connection_error.is_retryable());

    let transmission_error = ZerobusError::TransmissionError("test".to_string());
    assert!(transmission_error.is_retryable());

    let config_error = ZerobusError::ConfigurationError("test".to_string());
    assert!(!config_error.is_retryable());

    let auth_error = ZerobusError::AuthenticationError("test".to_string());
    assert!(!auth_error.is_retryable());
}

#[test]
fn test_error_is_token_expired() {
    let auth_error = ZerobusError::AuthenticationError("token expired".to_string());
    assert!(auth_error.is_token_expired());

    let config_error = ZerobusError::ConfigurationError("test".to_string());
    assert!(!config_error.is_token_expired());
}

#[test]
fn test_error_display() {
    let error = ZerobusError::ConfigurationError("test error".to_string());
    let error_str = format!("{}", error);
    assert!(error_str.contains("Configuration error"));
    assert!(error_str.contains("test error"));
}

#[test]
fn test_error_clone() {
    let error = ZerobusError::ConnectionError("test".to_string());
    let cloned = error.clone();
    assert!(matches!(cloned, ZerobusError::ConnectionError(_)));
}

#[test]
fn test_all_error_variants() {
    let _config = ZerobusError::ConfigurationError("config".to_string());
    let _auth = ZerobusError::AuthenticationError("auth".to_string());
    let _conn = ZerobusError::ConnectionError("conn".to_string());
    let _conv = ZerobusError::ConversionError("conv".to_string());
    let _trans = ZerobusError::TransmissionError("trans".to_string());
    let _retry = ZerobusError::RetryExhausted("retry".to_string());
    let _token = ZerobusError::TokenRefreshError("token".to_string());
}

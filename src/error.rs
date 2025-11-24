//! Error types for the Zerobus SDK Wrapper
//!
//! This module defines all error types used throughout the wrapper,
//! providing clear, actionable error messages for developers.

use thiserror::Error;

/// Error type for wrapper operations
///
/// All errors are descriptive and actionable, providing sufficient
/// information for developers to diagnose and resolve issues.
#[derive(Debug, Clone, Error)]
pub enum ZerobusError {
    /// Invalid configuration error
    ///
    /// Occurs when configuration values are invalid or missing required fields.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Authentication failure error
    ///
    /// Occurs when authentication with Zerobus fails (invalid credentials,
    /// expired tokens, etc.).
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Network/connection error
    ///
    /// Occurs when network connectivity is lost or connection to Zerobus fails.
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Arrow to Protobuf conversion failure
    ///
    /// Occurs when Arrow RecordBatch data cannot be converted to Protobuf format.
    #[error("Conversion error: {0}")]
    ConversionError(String),

    /// Data transmission failure
    ///
    /// Occurs when data transmission to Zerobus fails.
    #[error("Transmission error: {0}")]
    TransmissionError(String),

    /// All retry attempts exhausted
    ///
    /// Occurs when all retry attempts for transient failures have been exhausted.
    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),

    /// Token refresh failure
    ///
    /// Occurs when authentication token refresh fails.
    #[error("Token refresh error: {0}")]
    TokenRefreshError(String),
}

impl ZerobusError {
    /// Check if the error is retryable
    ///
    /// Returns true for transient errors that should be retried:
    /// - ConnectionError
    /// - TransmissionError (if transient)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ZerobusError::ConnectionError(_) | ZerobusError::TransmissionError(_)
        )
    }

    /// Check if the error indicates token expiration
    ///
    /// Returns true if the error suggests the authentication token has expired.
    pub fn is_token_expired(&self) -> bool {
        matches!(self, ZerobusError::AuthenticationError(_))
    }
}

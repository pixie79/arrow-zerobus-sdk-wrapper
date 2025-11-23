//! Authentication and token refresh
//!
//! This module handles authentication with Zerobus and automatic token refresh.

use crate::error::ZerobusError;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// OAuth2 token response
#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
}

/// Check if an error indicates token expiration
///
/// # Arguments
///
/// * `error` - Error to check
///
/// # Returns
///
/// Returns true if the error suggests token expiration.
pub fn is_token_expired_error(error: &ZerobusError) -> bool {
    matches!(error, ZerobusError::AuthenticationError(_))
}

/// Refresh authentication token using OAuth2 client credentials flow
///
/// Refreshes the OAuth2 token using the provided credentials by calling
/// the Unity Catalog OAuth endpoint.
///
/// # Arguments
///
/// * `unity_catalog_url` - Unity Catalog URL for OAuth (e.g., https://workspace.cloud.databricks.com)
/// * `client_id` - OAuth2 client ID
/// * `client_secret` - OAuth2 client secret
///
/// # Returns
///
/// Returns new access token, or error if refresh fails.
///
/// # Errors
///
/// Returns `TokenRefreshError` if token refresh fails.
pub async fn refresh_token(
    unity_catalog_url: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String, ZerobusError> {
    info!("Refreshing authentication token from {}", unity_catalog_url);

    // Build OAuth token endpoint URL
    let token_url = if unity_catalog_url.ends_with('/') {
        format!("{}oidc/v1/token", unity_catalog_url)
    } else {
        format!("{}/oidc/v1/token", unity_catalog_url)
    };

    debug!("Token endpoint: {}", token_url);

    // Prepare OAuth2 client credentials request
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| {
            ZerobusError::TokenRefreshError(format!("Failed to create HTTP client: {}", e))
        })?;

    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    // Make OAuth2 token request
    let response = client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            ZerobusError::TokenRefreshError(format!(
                "Failed to send token refresh request: {}",
                e
            ))
        })?;

    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        
        warn!(
            "Token refresh failed with status {}: {}",
            status, error_text
        );

        return Err(ZerobusError::TokenRefreshError(format!(
            "Token refresh failed with status {}: {}",
            status, error_text
        )));
    }

    // Parse token response
    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| {
            ZerobusError::TokenRefreshError(format!(
                "Failed to parse token response: {}",
                e
            ))
        })?;

    debug!(
        "Token refresh successful, expires_in: {:?}",
        token_response.expires_in
    );

    Ok(token_response.access_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_token_expired_error() {
        let auth_error = ZerobusError::AuthenticationError("token expired".to_string());
        assert!(is_token_expired_error(&auth_error));

        let config_error = ZerobusError::ConfigurationError("test".to_string());
        assert!(!is_token_expired_error(&config_error));
    }

    #[tokio::test]
    #[ignore] // Requires actual OAuth endpoint
    async fn test_refresh_token_integration() {
        // This test requires actual OAuth credentials and endpoint
        // It's marked as ignored and should be run manually with real credentials
        let result = refresh_token(
            "https://test.cloud.databricks.com",
            "test_client_id",
            "test_client_secret",
        )
        .await;

        // Will fail without real credentials, but tests the code path
        assert!(result.is_err());
    }
}

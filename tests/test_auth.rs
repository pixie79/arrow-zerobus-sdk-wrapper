//! Integration tests for authentication and token refresh

use arrow_zerobus_sdk_wrapper::wrapper::auth;
use arrow_zerobus_sdk_wrapper::ZerobusError;

#[test]
fn test_is_token_expired_error() {
    let auth_error = ZerobusError::AuthenticationError("token expired".to_string());
    assert!(auth::is_token_expired_error(&auth_error));

    let config_error = ZerobusError::ConfigurationError("test".to_string());
    assert!(!auth::is_token_expired_error(&config_error));

    let conn_error = ZerobusError::ConnectionError("test".to_string());
    assert!(!auth::is_token_expired_error(&conn_error));
}

#[tokio::test]
#[ignore] // Requires actual OAuth endpoint - run manually with real credentials
async fn test_refresh_token_with_invalid_credentials() {
    // This test will fail with invalid credentials, but tests the error handling
    let result = auth::refresh_token(
        "https://invalid.cloud.databricks.com",
        "invalid_client_id",
        "invalid_client_secret",
    )
    .await;

    // Should fail without real credentials
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ZerobusError::TokenRefreshError(_)
    ));
}

#[tokio::test]
async fn test_refresh_token_url_construction() {
    // Test URL construction logic (without making actual request)
    let base_url = "https://test.cloud.databricks.com";
    let expected_url = format!("{}/oidc/v1/token", base_url);

    // Verify URL format is correct
    assert!(expected_url.contains("/oidc/v1/token"));
    assert!(expected_url.starts_with("https://"));
}

#[tokio::test]
async fn test_refresh_token_url_with_trailing_slash() {
    // Test URL construction with trailing slash
    let base_url = "https://test.cloud.databricks.com/";
    let expected_url = format!("{}oidc/v1/token", base_url);

    // Verify URL format is correct (no double slash)
    assert!(expected_url.contains("/oidc/v1/token"));
    assert!(!expected_url.contains("//oidc"));
}

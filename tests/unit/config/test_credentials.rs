//! Unit tests for secure credential handling
//!
//! Tests to verify that credentials are stored as SecretString
//! and are not exposed in debug output or logs

use arrow_zerobus_sdk_wrapper::config::WrapperConfiguration;
use secrecy::{ExposeSecret, SecretString};

#[test]
fn test_credentials_stored_as_secret_string() {
    // Verify that client_id and client_secret are stored as SecretString
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
    );

    // Verify credentials are Some(SecretString)
    assert!(
        config.client_id.is_some(),
        "client_id should be set"
    );
    assert!(
        config.client_secret.is_some(),
        "client_secret should be set"
    );

    // Verify we can expose the secret when needed
    if let Some(client_id) = &config.client_id {
        let exposed = client_id.expose_secret();
        assert_eq!(exposed, "test_client_id");
    }

    if let Some(client_secret) = &config.client_secret {
        let exposed = client_secret.expose_secret();
        assert_eq!(exposed, "test_client_secret");
    }
}

#[test]
fn test_credentials_not_in_debug_output() {
    // Verify that credentials are not exposed in Debug output
    // SecretString's Debug implementation should not expose the secret
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        "sensitive_client_id".to_string(),
        "sensitive_client_secret".to_string(),
    );

    let debug_output = format!("{:?}", config);
    
    // The debug output should not contain the actual secret values
    // SecretString's Debug impl shows "<redacted>" or similar
    assert!(
        !debug_output.contains("sensitive_client_id"),
        "Debug output should not contain client_id: {}",
        debug_output
    );
    assert!(
        !debug_output.contains("sensitive_client_secret"),
        "Debug output should not contain client_secret: {}",
        debug_output
    );
}

#[test]
fn test_credentials_with_secret_string_direct() {
    // Test creating config with SecretString directly
    let client_id = SecretString::from("direct_client_id".to_string());
    let client_secret = SecretString::from("direct_client_secret".to_string());

    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        client_id.expose_secret().to_string(),
        client_secret.expose_secret().to_string(),
    );

    assert!(config.client_id.is_some());
    assert!(config.client_secret.is_some());
}

#[test]
fn test_credentials_optional() {
    // Test that credentials are optional
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    );

    assert!(config.client_id.is_none());
    assert!(config.client_secret.is_none());
}

#[test]
fn test_credentials_expose_secret_only_when_needed() {
    // Verify that expose_secret() is the only way to access the secret
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        "test_id".to_string(),
        "test_secret".to_string(),
    );

    // The only way to access the secret is through expose_secret()
    // This ensures we have explicit control over when secrets are exposed
    if let Some(client_id) = &config.client_id {
        let _exposed = client_id.expose_secret();
        // After this point, the exposed string is in memory
        // but we've made an explicit decision to expose it
    }
}

#[test]
fn test_secret_string_from_string() {
    // Test that SecretString can be created from String
    let secret = SecretString::from("my_secret".to_string());
    assert_eq!(secret.expose_secret(), "my_secret");
}

#[test]
fn test_secret_string_clone() {
    // Test that SecretString can be cloned
    // (This is important for configuration cloning)
    let secret1 = SecretString::from("clone_test".to_string());
    let secret2 = secret1.clone();
    
    assert_eq!(secret1.expose_secret(), secret2.expose_secret());
}

#[test]
fn test_config_with_credentials_returns_secret_string() {
    // Verify that with_credentials stores values as SecretString
    let config = WrapperConfiguration::new(
        "https://test.cloud.databricks.com".to_string(),
        "test_table".to_string(),
    )
    .with_credentials(
        "id123".to_string(),
        "secret456".to_string(),
    );

    // Verify the types are correct
    match &config.client_id {
        Some(id) => {
            // This should compile - SecretString has expose_secret()
            let _ = id.expose_secret();
        }
        None => panic!("client_id should be set"),
    }

    match &config.client_secret {
        Some(secret) => {
            // This should compile - SecretString has expose_secret()
            let _ = secret.expose_secret();
        }
        None => panic!("client_secret should be set"),
    }
}


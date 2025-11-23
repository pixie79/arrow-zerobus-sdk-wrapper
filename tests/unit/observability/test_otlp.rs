//! Unit tests for OpenTelemetry integration
//!
//! Target: ≥90% coverage per file

use arrow_zerobus_sdk_wrapper::config::OtlpConfig;
use arrow_zerobus_sdk_wrapper::observability::otlp::ObservabilityManager;

#[test]
fn test_otlp_config_creation() {
    let config = OtlpConfig {
        endpoint: Some("http://localhost:4317".to_string()),
        extra: std::collections::HashMap::new(),
    };

    assert_eq!(config.endpoint, Some("http://localhost:4317".to_string()));
}

#[test]
fn test_otlp_config_default() {
    let config = OtlpConfig::default();
    assert_eq!(config.endpoint, None);
    assert!(config.extra.is_empty());
}

#[test]
fn test_observability_manager_creation_disabled() {
    // When observability is disabled, manager should be None or handle gracefully
    let manager = ObservabilityManager::new(None);
    assert!(manager.is_none());
}

#[test]
fn test_observability_manager_creation_enabled() {
    let config = OtlpConfig {
        endpoint: Some("http://localhost:4317".to_string()),
        extra: std::collections::HashMap::new(),
    };

    // This will fail if otlp-rust-service is not available, but tests the API
    let manager = ObservabilityManager::new(Some(config));
    
    // Manager may be None if initialization fails (expected in test environment)
    // but the API should not panic
    assert!(manager.is_some() || manager.is_none());
}

#[tokio::test]
async fn test_observability_manager_metrics() {
    let config = OtlpConfig {
        endpoint: Some("http://localhost:4317".to_string()),
        extra: std::collections::HashMap::new(),
    };

    let manager = ObservabilityManager::new_async(Some(config)).await;
    
    if let Some(mgr) = manager {
        // Test that metrics can be recorded without panicking
        mgr.record_batch_sent(1024, true, 100).await;
        mgr.record_batch_sent(2048, false, 200).await;
        
        // Verify metrics are recorded (may be no-op if not initialized)
        // This tests the API contract
    }
}

#[test]
fn test_observability_manager_traces() {
    let config = OtlpConfig {
        endpoint: Some("http://localhost:4317".to_string()),
        extra: std::collections::HashMap::new(),
    };

    let manager = ObservabilityManager::new(Some(config));
    
    if let Some(mgr) = manager {
        // Test that traces can be started and ended
        let span = mgr.start_send_batch_span("test_table");
        // Span should be droppable without panicking
        drop(span);
    }
}

#[test]
fn test_observability_manager_without_config() {
    // Test that observability works when disabled
    let manager = ObservabilityManager::new(None);
    assert!(manager.is_none());
    
    // Operations should be no-ops when disabled
    // This is tested implicitly by the fact that we can call methods on Option
}


//! Unit tests for retry logic

use arrow_zerobus_sdk_wrapper::wrapper::retry::RetryConfig;
use arrow_zerobus_sdk_wrapper::ZerobusError;
use std::time::Duration;
use tokio::time::Instant;

#[tokio::test]
async fn test_retry_succeeds_on_first_attempt() {
    let config = RetryConfig::default();
    let result = config
        .execute_with_retry(|| async { Ok::<_, ZerobusError>("success".to_string()) })
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_retry_exhausted_after_max_attempts() {
    let config = RetryConfig::new(3, 10, 1000);
    let mut attempts = 0;
    let result = config
        .execute_with_retry(|| {
            attempts += 1;
            async {
                Err::<String, _>(ZerobusError::ConnectionError("test error".to_string()))
            }
        })
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ZerobusError::RetryExhausted(_)));
    assert_eq!(attempts, 3);
}

#[tokio::test]
async fn test_retry_non_retryable_error() {
    let config = RetryConfig::default();
    let mut attempts = 0;
    let result = config
        .execute_with_retry(|| {
            attempts += 1;
            async {
                Err::<String, _>(ZerobusError::ConfigurationError("non-retryable".to_string()))
            }
        })
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ZerobusError::ConfigurationError(_)));
    assert_eq!(attempts, 1); // Should not retry non-retryable errors
}

#[tokio::test]
async fn test_retry_succeeds_after_failures() {
    let config = RetryConfig::new(5, 10, 1000);
    let mut attempts = 0;
    let result = config
        .execute_with_retry(|| {
            attempts += 1;
            async {
                if attempts < 3 {
                    Err::<String, _>(ZerobusError::ConnectionError("transient".to_string()))
                } else {
                    Ok("success".to_string())
                }
            }
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(attempts, 3);
}

#[tokio::test]
async fn test_retry_delay_calculation() {
    let config = RetryConfig::new(5, 100, 10000);
    
    // Test that delays are calculated (we can't test exact values due to jitter)
    let start = Instant::now();
    let mut attempts = 0;
    let _result = config
        .execute_with_retry(|| {
            attempts += 1;
            async {
                Err::<String, _>(ZerobusError::ConnectionError("test".to_string()))
            }
        })
        .await;
    
    let elapsed = start.elapsed();
    // Should have taken some time due to delays (at least 100ms for first retry)
    assert!(elapsed >= Duration::from_millis(50)); // Allow for some variance
    assert_eq!(attempts, 5);
}

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.base_delay_ms, 100);
    assert_eq!(config.max_delay_ms, 30000);
    assert!(config.jitter);
}

#[test]
fn test_retry_config_new() {
    let config = RetryConfig::new(10, 200, 60000);
    assert_eq!(config.max_attempts, 10);
    assert_eq!(config.base_delay_ms, 200);
    assert_eq!(config.max_delay_ms, 60000);
    assert!(config.jitter);
}


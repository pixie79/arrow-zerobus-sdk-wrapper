//! Integration tests for retry logic

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
            async { Err::<String, _>(ZerobusError::ConnectionError("test error".to_string())) }
        })
        .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ZerobusError::RetryExhausted(_)
    ));
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
                Err::<String, _>(ZerobusError::ConfigurationError(
                    "non-retryable".to_string(),
                ))
            }
        })
        .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ZerobusError::ConfigurationError(_)
    ));
    assert_eq!(attempts, 1); // Should not retry non-retryable errors
}

#[tokio::test]
async fn test_retry_succeeds_after_failures() {
    let config = RetryConfig::new(5, 10, 1000);
    let attempts = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempts_clone = attempts.clone();
    let result = config
        .execute_with_retry(|| {
            let attempts = attempts_clone.clone();
            async move {
                let mut count = attempts.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);

                if current < 3 {
                    Err::<String, _>(ZerobusError::ConnectionError("transient".to_string()))
                } else {
                    Ok("success".to_string())
                }
            }
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(*attempts.lock().unwrap(), 3);
}

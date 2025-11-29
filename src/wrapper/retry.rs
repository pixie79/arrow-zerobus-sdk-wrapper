//! Retry logic with exponential backoff and jitter
//!
//! This module implements retry logic with exponential backoff and full jitter
//! for handling transient failures.

use crate::error::ZerobusError;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay in milliseconds for exponential backoff
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Enable jitter in backoff calculation (default: true)
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 100,
            max_delay_ms: 30000,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            base_delay_ms,
            max_delay_ms,
            jitter: true,
        }
    }

    /// Execute a function with retry logic
    ///
    /// Retries the function with exponential backoff + jitter if it returns
    /// a retryable error. Returns the result if successful, or the last error
    /// if all retries are exhausted.
    ///
    /// # Arguments
    ///
    /// * `f` - Async function to execute
    ///
    /// # Returns
    ///
    /// Returns the result of the function if successful, or `RetryExhausted` error
    /// if all retry attempts are exhausted.
    pub async fn execute_with_retry<F, Fut, T>(&self, f: F) -> Result<T, ZerobusError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, ZerobusError>>,
    {
        let (result, _) = self.execute_with_retry_tracked(f).await;
        result
    }

    /// Execute a function with retry logic and track attempt count
    ///
    /// Retries the function with exponential backoff + jitter if it returns
    /// a retryable error. Returns both the result and the number of attempts made.
    ///
    /// # Arguments
    ///
    /// * `f` - Async function to execute
    ///
    /// # Returns
    ///
    /// Returns a tuple of (result, attempts) where:
    /// - `result`: The result of the function if successful, or `RetryExhausted` error
    ///   if all retry attempts are exhausted.
    /// - `attempts`: The number of attempts made (1-indexed, so 1 means first attempt succeeded)
    pub async fn execute_with_retry_tracked<F, Fut, T>(
        &self,
        mut f: F,
    ) -> (Result<T, ZerobusError>, u32)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, ZerobusError>>,
    {
        let mut last_error = None;

        for attempt in 0..self.max_attempts {
            let attempt_number = attempt + 1; // 1-indexed
            match f().await {
                Ok(result) => return (Ok(result), attempt_number),
                Err(e) => {
                    last_error = Some(e.clone());

                    // Check if error is retryable
                    if !e.is_retryable() {
                        return (Err(e), attempt_number);
                    }

                    // Don't sleep after the last attempt
                    if attempt < self.max_attempts - 1 {
                        let delay = self.calculate_delay(attempt);
                        sleep(delay).await;
                    }
                }
            }
        }

        // All retries exhausted
        (
            Err(ZerobusError::RetryExhausted(format!(
                "All {} retry attempts exhausted. Last error: {}",
                self.max_attempts,
                last_error
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ))),
            self.max_attempts,
        )
    }

    /// Calculate delay for the given attempt number
    ///
    /// Uses exponential backoff: delay = base_delay * (2 ^ attempt_number)
    /// With full jitter: random delay between 0 and calculated exponential delay
    ///
    /// # Arguments
    ///
    /// * `attempt` - Current attempt number (0-indexed)
    ///
    /// # Returns
    ///
    /// Returns the delay duration for this attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        // Calculate exponential backoff: base_delay * 2^attempt
        let exponential_delay_ms = self.base_delay_ms.saturating_mul(1 << attempt.min(20));

        // Cap at max_delay_ms
        let capped_delay_ms = exponential_delay_ms.min(self.max_delay_ms);

        // Apply full jitter if enabled
        let delay_ms = if self.jitter {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..=capped_delay_ms)
        } else {
            capped_delay_ms
        };

        Duration::from_millis(delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

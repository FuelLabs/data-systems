use std::{future::Future, sync::LazyLock, time::Duration};

use crate::errors::ConsumerError;

static DB_TIMEOUT: LazyLock<Duration> = LazyLock::new(|| {
    Duration::from_secs(
        dotenvy::var("DB_TIMEOUT")
            .unwrap_or("240".to_string())
            .parse::<u64>()
            .unwrap(),
    )
});

#[derive(Debug)]
pub struct RetryService {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub timeout: Duration,
}

impl Default for RetryService {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            timeout: *DB_TIMEOUT,
        }
    }
}

impl RetryService {
    fn get_delay_for_attempt(&self, attempt: u32) -> Duration {
        self.initial_delay * 2u32.pow(attempt.saturating_sub(1))
    }

    pub async fn with_retry<F, Fut, T>(
        &self,
        operation_name: &str,
        f: F,
    ) -> Result<T, ConsumerError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, ConsumerError>>,
    {
        let mut attempt = 0;
        let mut last_error = None;
        while attempt < self.max_retries {
            let result = tokio::time::timeout(self.timeout, f()).await;
            match result {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(e)) => {
                    if !Self::is_retryable_error(&e) {
                        tracing::warn!(
                            error = ?e,
                            operation = operation_name,
                            "Non-retryable error encountered"
                        );
                        return Err(e);
                    }

                    tracing::warn!(
                        error = ?e,
                        attempt = attempt + 1,
                        operation = operation_name,
                        "Operation failed, retrying"
                    );
                    last_error = Some(e);
                }
                Err(_) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        operation = operation_name,
                        "Operation timed out, retrying"
                    );
                    last_error = Some(ConsumerError::DatabaseTimeout);
                }
            }

            attempt += 1;
            if attempt < self.max_retries {
                let delay = self.get_delay_for_attempt(attempt);
                tracing::debug!(
                    delay_ms = ?delay.as_millis(),
                    operation = operation_name,
                    "Waiting before retry"
                );
                tokio::time::sleep(delay).await;
            }
        }

        tracing::error!(
            error = ?last_error,
            operation = operation_name,
            "Operation failed after all retry attempts"
        );
        Err(last_error.unwrap())
    }

    fn is_retryable_error(error: &ConsumerError) -> bool {
        matches!(error, ConsumerError::DatabaseTimeout | ConsumerError::Db(_))
    }
}

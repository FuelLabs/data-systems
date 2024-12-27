use std::{future::Future, time::Duration};

use tracing;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
        }
    }
}

pub async fn with_retry<T, Fut, F, E>(
    config: &RetryConfig,
    operation_name: &str,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut last_error = None;
    while attempt < config.max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                attempt += 1;

                if attempt < config.max_retries {
                    let backoff =
                        config.initial_backoff * 2u32.pow(attempt - 1);
                    tracing::warn!(
                        "{} failed, attempt {}/{}: {}. Retrying in {:?}",
                        operation_name,
                        attempt,
                        config.max_retries,
                        last_error.as_ref().unwrap(),
                        backoff
                    );
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    use super::*;

    #[tokio::test]
    async fn test_retry_mechanism() {
        let config = RetryConfig {
            max_retries: 3,
            initial_backoff: Duration::from_millis(10), /* Shorter duration for tests */
        };

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let result: Result<(), String> = with_retry(&config, "test", || {
            let value = counter_clone.clone();
            async move {
                let attempt = value.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    // Fail first two attempts
                    Err("Simulated failure".to_string())
                } else {
                    // Succeed on third attempt
                    Ok(())
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        let config = RetryConfig {
            max_retries: 3,
            initial_backoff: Duration::from_millis(10),
        };

        let result: Result<(), String> =
            with_retry(&config, "test", || async {
                Err("Always fails".to_string())
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Always fails");
    }
}

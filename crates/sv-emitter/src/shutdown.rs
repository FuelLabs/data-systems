use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[derive(Error, Debug)]
pub enum ShutdownError {
    #[error("Shutdown Operation: {0}")]
    Cancelled(#[from] Box<dyn std::error::Error + Send + Sync>),
}

type ShutdownVec = Vec<Box<dyn Send + FnOnce() + 'static>>;

#[derive(Clone)]
pub struct ShutdownController {
    token: CancellationToken,
    shutdown_handlers: Arc<Mutex<ShutdownVec>>,
}

impl Default for ShutdownController {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownController {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            shutdown_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn token(&self) -> &CancellationToken {
        &self.token
    }

    pub fn spawn_signal_handler(self: Arc<Self>) -> Arc<Self> {
        tokio::spawn({
            let shutdown = self.clone();
            async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for ctrl+c");
                tracing::info!("Received shutdown signal");
                shutdown.initiate_shutdown();
            }
        });
        self
    }

    pub fn on_shutdown<F>(&self, handler: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.shutdown_handlers
            .lock()
            .unwrap()
            .push(Box::new(handler));
    }

    pub fn initiate_shutdown(&self) {
        tracing::info!("Initiating graceful shutdown...");

        // Execute all shutdown handlers
        let handlers =
            std::mem::take(&mut *self.shutdown_handlers.lock().unwrap());
        for handler in handlers {
            handler();
        }

        self.token.cancel();
    }

    pub async fn run_with_cancellation<F, Fut, T>(
        &self,
        f: F,
    ) -> Result<T, ShutdownError>
    where
        F: FnOnce(CancellationToken) -> Fut,
        Fut: Future<Output = Result<T, ShutdownError>>,
    {
        tokio::select! {
            _ = self.token.cancelled() => {
                tracing::info!("Shutdown initiated, waiting for tasks to complete...");
                Err(ShutdownError::Cancelled(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Task cancelled during shutdown"
                ))))
            }
            result = f(self.token.clone()) => {
                if let Err(e) = &result {
                    tracing::error!("Task error: {:?}", e);
                }
                result
            }
        }
    }

    pub fn is_shutdown_initiated(&self) -> bool {
        self.token.is_cancelled()
    }

    pub async fn wait_for_shutdown(&self) {
        self.token.cancelled().await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use super::*;

    #[tokio::test]
    async fn test_manual_shutdown() {
        let controller = ShutdownController::new();
        assert!(!controller.is_shutdown_initiated());

        controller.initiate_shutdown();
        assert!(controller.is_shutdown_initiated());
    }

    #[tokio::test]
    async fn test_normal_completion() {
        let controller = ShutdownController::new();

        let result = controller
            .run_with_cancellation(|_| async move { Ok(42) })
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_cancelled_completion() {
        let controller = ShutdownController::new();

        tokio::spawn({
            let controller = controller.clone();
            async move {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                controller.initiate_shutdown();
            }
        });

        let result = controller
            .run_with_cancellation(|_| async move {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                Ok(42)
            })
            .await;

        assert!(matches!(result, Err(ShutdownError::Cancelled(_))));
    }

    #[tokio::test]
    async fn test_shutdown_handlers() {
        let controller = ShutdownController::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        controller.on_shutdown(move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        controller.initiate_shutdown();
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}

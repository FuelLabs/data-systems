use std::{sync::Arc, time::Duration};

use fuel_message_broker::MessageBroker;
use tokio_util::sync::CancellationToken;

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

pub async fn shutdown_broker_with_timeout(broker: &Arc<dyn MessageBroker>) {
    let _ = tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, async {
        tracing::info!("Flushing in-flight messages to broker ...");
        match broker.flush().await {
            Ok(_) => {
                tracing::info!("Flushed all streams successfully!");
            }
            Err(e) => {
                tracing::error!("Failed to flush all streams: {:?}", e);
            }
        }
    })
    .await;
}

#[derive(Clone)]
pub struct ShutdownController {
    token: CancellationToken,
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

    pub fn initiate_shutdown(&self) {
        tracing::info!("Initiating graceful shutdown...");
        self.token.cancel();
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
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_manual_shutdown() {
        let controller = ShutdownController::new();
        assert!(
            !controller.is_shutdown_initiated(),
            "Controller should not be shutdown initially"
        );

        controller.initiate_shutdown();
        assert!(
            controller.is_shutdown_initiated(),
            "Controller should be shutdown after initiation"
        );
    }

    #[tokio::test]
    async fn test_wait_for_shutdown_timeout() {
        let controller = ShutdownController::new();

        let timeout = Duration::from_millis(50);
        let result =
            tokio::time::timeout(timeout, controller.wait_for_shutdown()).await;

        assert!(
            result.is_err(),
            "wait_for_shutdown should not complete without initiation"
        );
    }

    #[tokio::test]
    async fn test_clone_behavior() {
        let controller = ShutdownController::new();
        let cloned = controller.clone();

        // Initiate shutdown from clone
        cloned.initiate_shutdown();

        assert!(
            controller.is_shutdown_initiated(),
            "Original should be shutdown"
        );
        assert!(cloned.is_shutdown_initiated(), "Clone should be shutdown");
    }
}

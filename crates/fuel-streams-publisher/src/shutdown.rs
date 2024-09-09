use std::time::Duration;

use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{broadcast, OnceCell},
};
use tracing::info;

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Debug)]
pub struct StopHandle {
    sender: broadcast::Sender<()>,
    receiver: broadcast::Receiver<()>,
    shutdown_initiated: OnceCell<()>,
}

impl Default for StopHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl StopHandle {
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel::<()>(1);
        Self {
            sender,
            receiver,
            shutdown_initiated: OnceCell::new(),
        }
    }

    pub fn spawn_signal_listener(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut sigint =
                signal(SignalKind::interrupt()).expect("shutdown_listener");
            let mut sigterm =
                signal(SignalKind::terminate()).expect("shutdown_listener");

            tokio::select! {
                _ = sigint.recv() => {
                    info!("Received SIGINT ...");
                    let _ = sender.send(());
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM ...");
                    let _ = sender.send(());
                }
            }
        });
    }

    pub async fn wait_for_signal(&mut self) -> bool {
        self.receiver.recv().await.is_ok() && self.initiate_shutdown()
    }

    fn initiate_shutdown(&self) -> bool {
        self.shutdown_initiated.set(()).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{timeout, Duration};

    use super::*;

    #[tokio::test]
    async fn test_stop_handle_receives_signal() {
        let mut stop_handle = StopHandle::new();
        stop_handle.spawn_signal_listener();

        // Simulate sending the shutdown signal
        let _ = stop_handle.sender.send(());

        // Assert that the signal is received correctly
        assert!(
            stop_handle.wait_for_signal().await,
            "StopHandle should receive the signal and return true"
        );
    }

    #[tokio::test]
    async fn test_stop_handle_only_shutdowns_once() {
        let mut stop_handle = StopHandle::new();
        stop_handle.spawn_signal_listener();

        // Simulate sending the shutdown signal
        let _ = stop_handle.sender.send(());

        // Assert that the shutdown is initiated once
        assert!(
            stop_handle.wait_for_signal().await,
            "StopHandle should receive the first signal and initiate shutdown"
        );

        // Now, simulate sending another shutdown signal
        let _ = stop_handle.sender.send(());

        // Assert that subsequent shutdowns are ignored
        assert!(
            !stop_handle.wait_for_signal().await,
            "StopHandle should not initiate shutdown again"
        );
    }

    #[tokio::test]
    async fn test_stop_handle_no_signal_received() {
        let mut stop_handle = StopHandle::new();
        stop_handle.spawn_signal_listener();

        // We will wait for a signal that is never sent
        let result =
            timeout(Duration::from_millis(100), stop_handle.wait_for_signal())
                .await;

        // Assert that the timeout occurred because no signal was sent
        assert!(
            result.is_err(),
            "No signal was sent, so wait_for_signal should timeout"
        );
    }
}

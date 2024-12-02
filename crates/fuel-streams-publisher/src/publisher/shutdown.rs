use std::{sync::Arc, time::Duration};

use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{broadcast, OnceCell},
};

// TODO: move into publisher module along with subjects

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

// First, let's create a ShutdownToken that can be shared
#[derive(Debug, Clone)]
pub struct ShutdownToken {
    receiver: Arc<broadcast::Receiver<()>>,
}

impl ShutdownToken {
    pub async fn wait_for_shutdown(&self) -> bool {
        let mut rx = self.receiver.resubscribe();
        rx.recv().await.is_ok()
    }
}

#[derive(Debug)]
pub struct ShutdownController {
    sender: broadcast::Sender<()>,
    token: ShutdownToken,
    shutdown_initiated: OnceCell<()>,
}

impl Default for ShutdownController {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownController {
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel(1);
        let token = ShutdownToken {
            receiver: Arc::new(receiver),
        };

        Self {
            sender,
            token,
            shutdown_initiated: OnceCell::new(),
        }
    }
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn get_token(&self) -> ShutdownToken {
        self.token.clone()
    }

    pub fn spawn_signal_listener(self: Arc<Self>) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut sigint =
                signal(SignalKind::interrupt()).expect("shutdown_listener");
            let mut sigterm =
                signal(SignalKind::terminate()).expect("shutdown_listener");

            tokio::select! {
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT ...");
                    let _ = sender.send(());
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM ...");
                    let _ = sender.send(());
                }
            }
        });
    }

    pub fn initiate_shutdown(
        &self,
    ) -> Result<usize, broadcast::error::SendError<()>> {
        if self.shutdown_initiated.set(()).is_ok() {
            self.sender.send(())
        } else {
            Ok(0) // Shutdown already initiated
        }
    }
}

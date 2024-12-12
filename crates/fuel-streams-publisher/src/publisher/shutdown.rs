use std::time::Duration;

use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{broadcast, OnceCell},
};

// TODO: move into publisher module along with subjects

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

// First, let's create a ShutdownToken that can be shared
#[derive(Debug)]
pub struct ShutdownToken {
    receiver: broadcast::Receiver<()>,
}

impl ShutdownToken {
    pub async fn wait_for_shutdown(&mut self) -> bool {
        self.receiver.recv().await.is_ok()
    }
}

#[derive(Debug, Clone)]
pub struct ShutdownController {
    sender: broadcast::Sender<()>,
    shutdown_initiated: OnceCell<()>,
}

impl ShutdownController {
    pub fn spawn_signal_listener(&self) {
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

pub fn get_controller_and_token() -> (ShutdownController, ShutdownToken) {
    let (sender, receiver) = broadcast::channel(1);

    (
        ShutdownController {
            sender,
            shutdown_initiated: OnceCell::new(),
        },
        ShutdownToken { receiver },
    )
}

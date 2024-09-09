use std::time::Duration;

use tokio::{
    signal::unix::{signal, SignalKind},
    sync::broadcast,
};
use tracing::info;

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn stop_signal(stop_tx: broadcast::Sender<()>) {
    let mut sigint =
        signal(SignalKind::interrupt()).expect("shutdown_listener");
    let mut sigterm =
        signal(SignalKind::terminate()).expect("shutdown_listener");
    tokio::select! {
        _ = sigint.recv() => {
            info!("Received SIGINT ...");
            let _ = stop_tx.send(());
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM ...");
            let _ = stop_tx.send(());
        }
    }
}

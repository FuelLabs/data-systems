use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use fuel_web_utils::api_key::{ApiKey, ApiKeyId};
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum ConnectionSignal {
    Ping,
    Timeout,
}

pub struct Connection {
    pub tx: Sender<ConnectionSignal>,
    pub last_heartbeat: Instant,
}

impl Connection {
    pub fn new(tx: Sender<ConnectionSignal>) -> Self {
        Self {
            tx,
            last_heartbeat: Instant::now(),
        }
    }

    pub fn update_time(&mut self) {
        self.last_heartbeat = Instant::now();
    }
}

type ConnectionsMap = DashMap<ApiKeyId, Connection>;

#[derive(Clone)]
pub struct ConnectionChecker {
    pub connections: Arc<ConnectionsMap>,
    ping_interval: Duration,
    heartbeat_timeout: Duration,
}

impl Default for ConnectionChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionChecker {
    pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(5);
    pub const DEFAULT_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);

    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            ping_interval: Self::DEFAULT_PING_INTERVAL,
            heartbeat_timeout: Self::DEFAULT_HEARTBEAT_TIMEOUT,
        }
    }

    pub fn with_ping_interval(self, ping_interval: Duration) -> Self {
        Self {
            ping_interval,
            ..self
        }
    }

    pub fn with_heartbeat_timeout(self, heartbeat_timeout: Duration) -> Self {
        Self {
            heartbeat_timeout,
            ..self
        }
    }

    pub async fn register(
        &self,
        api_key: &ApiKey,
        tx: Sender<ConnectionSignal>,
    ) {
        self.connections
            .insert(api_key.id().to_owned(), Connection::new(tx.clone()));
    }

    pub async fn unregister(&self, api_key: &ApiKey) {
        self.connections.remove(api_key.id());
    }

    pub async fn update_heartbeat(&self, api_key: &ApiKey) {
        if let Some(mut entry) = self.connections.get_mut(api_key.id()) {
            let entry = entry.value_mut();
            entry.update_time()
        }
    }

    pub async fn start(&self) {
        let ping_interval = self.ping_interval;
        let heartbeat_timeout = self.heartbeat_timeout;
        let connections = self.connections.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(ping_interval);
            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut to_remove = Vec::new();

                for entry in connections.iter() {
                    let api_key = entry.key();
                    let conn = entry.value();

                    // Send ping request via timeout_tx (handler will handle actual ping)
                    if conn.tx.send(ConnectionSignal::Ping).await.is_err() {
                        tracing::warn!(%api_key, "Failed to send ping request, channel closed");
                        to_remove.push(*api_key);
                        continue;
                    }

                    // Check heartbeat timeout
                    let duration = now.duration_since(conn.last_heartbeat);
                    if duration > heartbeat_timeout {
                        tracing::warn!(%api_key, timeout = ?heartbeat_timeout, "Client timeout; notifying handler");
                        if conn
                            .tx
                            .send(ConnectionSignal::Timeout)
                            .await
                            .is_err()
                        {
                            tracing::warn!(%api_key, "Failed to notify handler, channel closed");
                        }
                        to_remove.push(*api_key);
                    }
                }

                // Clean up timed-out or failed connections
                for api_key in to_remove {
                    connections.remove(&api_key);
                }

                if connections.is_empty() {
                    tracing::info!(
                        "No active connections, connection checker pausing"
                    );
                }
            }
        });
    }
}

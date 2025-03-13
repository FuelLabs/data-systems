use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait StateProvider: Send + Sync {
    async fn is_healthy(&self) -> bool;
    async fn get_health(&self) -> serde_json::Value;
    async fn get_metrics(&self) -> String;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefaultHealthResponse {
    pub uptime: u64,
    pub is_healthy: bool,
}

#[derive(Clone)]
pub struct DefaultServerState {
    pub start_time: Instant,
}

impl Default for DefaultServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultServerState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub async fn get_health(&self) -> DefaultHealthResponse {
        DefaultHealthResponse {
            uptime: self.uptime().as_secs(),
            is_healthy: true,
        }
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[async_trait]
impl StateProvider for DefaultServerState {
    async fn is_healthy(&self) -> bool {
        true
    }

    async fn get_health(&self) -> serde_json::Value {
        serde_json::to_value(DefaultHealthResponse {
            uptime: self.uptime().as_secs(),
            is_healthy: true,
        })
        .unwrap_or(serde_json::json!({}))
    }

    async fn get_metrics(&self) -> String {
        format!("uptime: {}s", self.uptime().as_secs())
    }
}

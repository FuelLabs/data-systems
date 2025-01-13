use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use fuel_message_broker::MessageBroker;
use fuel_web_utils::{server::state::StateProvider, telemetry::Telemetry};
use serde::{Deserialize, Serialize};

use crate::metrics::Metrics;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub uptime_secs: u64,
    pub is_healthy: bool,
}

pub struct ServerState {
    pub start_time: Instant,
    pub msg_broker: Arc<dyn MessageBroker>,
    pub telemetry: Arc<Telemetry<Metrics>>,
}

impl ServerState {
    pub fn new(
        msg_broker: Arc<dyn MessageBroker>,
        telemetry: Arc<Telemetry<Metrics>>,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            msg_broker,
            telemetry,
        }
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[async_trait]
impl StateProvider for ServerState {
    async fn is_healthy(&self) -> bool {
        self.msg_broker.is_healthy().await
    }

    async fn get_health(&self) -> serde_json::Value {
        let resp = HealthResponse {
            uptime_secs: self.uptime().as_secs(),
            is_healthy: self.is_healthy().await,
        };
        serde_json::to_value(resp).unwrap_or(serde_json::json!({}))
    }

    async fn get_metrics(&self) -> String {
        self.telemetry.get_metrics().await
    }
}

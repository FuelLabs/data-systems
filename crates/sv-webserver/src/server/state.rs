use std::{
    sync::Arc,
    time::{Duration, Instant},
};

// TODO: remove this once we have a proper broker implementation
use async_nats::jetstream::stream::State;
use async_trait::async_trait;
use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::FuelStreams;
use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::{server::state::StateProvider, telemetry::Telemetry};
use serde::{Deserialize, Serialize};

use crate::{config::Config, metrics::Metrics};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamInfo {
    consumers: Vec<String>,
    state: StreamState,
    stream_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct StreamState {
    /// The number of messages contained in this stream
    pub messages: u64,
    /// The number of bytes of all messages contained in this stream
    pub bytes: u64,
    /// The lowest sequence number still present in this stream
    #[serde(rename = "first_seq")]
    pub first_sequence: u64,
    /// The time associated with the oldest message still present in this stream
    #[serde(rename = "first_ts")]
    pub first_timestamp: i64,
    /// The last sequence number assigned to a message in this stream
    #[serde(rename = "last_seq")]
    pub last_sequence: u64,
    /// The time that the last message was received by this stream
    #[serde(rename = "last_ts")]
    pub last_timestamp: i64,
    /// The number of consumers configured to consume this stream
    pub consumer_count: usize,
}

impl From<State> for StreamState {
    fn from(state: State) -> Self {
        StreamState {
            messages: state.messages,
            bytes: state.bytes,
            first_sequence: state.first_sequence,
            first_timestamp: state.first_timestamp.unix_timestamp(),
            last_sequence: state.last_sequence,
            last_timestamp: state.last_timestamp.unix_timestamp(),
            consumer_count: state.consumer_count,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub uptime_secs: u64,
    pub is_healthy: bool,
    pub streams_info: Vec<StreamInfo>,
}

#[derive(Clone)]
pub struct ServerState {
    pub start_time: Instant,
    pub msg_broker: Arc<dyn MessageBroker>,
    pub fuel_streams: Arc<FuelStreams>,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub db: Arc<Db>,
    pub jwt_secret: String,
}

impl ServerState {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let msg_broker =
            MessageBrokerClient::Nats.start(&config.broker.url).await?;
        let db = Db::new(DbConnectionOpts {
            connection_str: config.db.url.clone(),
            ..Default::default()
        })
        .await?
        .arc();

        let fuel_streams = FuelStreams::new(&msg_broker, &db).await.arc();
        let metrics = Metrics::new_with_random_prefix()?;
        let telemetry = Telemetry::new(Some(metrics)).await?;
        telemetry.start().await?;

        Ok(Self {
            start_time: Instant::now(),
            msg_broker,
            fuel_streams,
            telemetry,
            db,
            jwt_secret: config.auth.jwt_secret.clone(),
        })
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
        let response = self.msg_broker.get_health_info().await;
        serde_json::to_value(response).unwrap_or(serde_json::json!({}))
    }

    async fn get_metrics(&self) -> String {
        self.telemetry.get_metrics().await
    }
}

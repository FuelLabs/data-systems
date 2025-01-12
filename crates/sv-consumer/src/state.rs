use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_nats::jetstream::stream::State;
use async_trait::async_trait;
use fuel_streams_core::{nats::NatsClient, FuelStreamsExt};
use fuel_web_utils::{server::state::StateProvider, telemetry::Telemetry};
use serde::{Deserialize, Serialize};

use crate::metrics::Metrics;

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

pub struct ServerState {
    pub start_time: Instant,
    pub nats_client: Arc<NatsClient>,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub fuel_streams: Arc<dyn FuelStreamsExt>,
}

impl ServerState {
    pub fn new(
        nats_client: Arc<NatsClient>,
        telemetry: Arc<Telemetry<Metrics>>,
        fuel_streams: Arc<dyn FuelStreamsExt>,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            nats_client,
            telemetry,
            fuel_streams,
        }
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[async_trait]
impl StateProvider for ServerState {
    async fn is_healthy(&self) -> bool {
        self.nats_client.is_connected()
    }

    async fn get_health(&self) -> serde_json::Value {
        let streams_info = self
            .fuel_streams
            .get_consumers_and_state()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|res| StreamInfo {
                consumers: res.1,
                state: res.2.into(),
                stream_name: res.0,
            })
            .collect::<Vec<StreamInfo>>();

        let resp = HealthResponse {
            uptime_secs: self.uptime().as_secs(),
            is_healthy: self.is_healthy().await,
            streams_info,
        };
        serde_json::to_value(resp).unwrap_or(serde_json::json!({}))
    }

    async fn get_metrics(&self) -> String {
        self.telemetry.get_metrics().await
    }
}

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_nats::jetstream::stream::State;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::Publisher;

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
    pub uptime: u64,
    pub streams_info: Vec<StreamInfo>,
}

#[derive(Clone)]
pub struct ServerState {
    pub publisher: Publisher,
    pub start_time: Instant,
    pub connection_count: Arc<RwLock<u32>>,
}

impl ServerState {
    pub async fn new(publisher: Publisher) -> Self {
        Self {
            publisher,
            start_time: Instant::now(),
            connection_count: Arc::new(RwLock::new(0)),
        }
    }
}

impl ServerState {
    pub fn is_healthy(&self) -> bool {
        if !self.publisher.fuel_core.is_started() {
            return false;
        }
        if !self.publisher.nats_client.is_connected() {
            return false;
        }
        true
    }

    pub async fn get_health(&self) -> HealthResponse {
        let streams_info = self
            .publisher
            .streams
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
        HealthResponse {
            uptime: self.uptime().as_secs(),
            streams_info,
        }
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

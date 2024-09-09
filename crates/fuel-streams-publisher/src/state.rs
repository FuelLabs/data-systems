use std::{
    sync::Arc,
    time::{Duration, Instant},
    vec,
};

use async_nats::jetstream::stream::State;
use fuel_core_bin::FuelService;
use fuel_streams_core::nats::{NatsClient, NatsClientOpts};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    metrics::PublisherMetrics,
    system::{PromSystemMetrics, System},
    Streams,
};

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
pub struct SharedState {
    pub fuel_service: Arc<FuelService>,
    pub nats_client: NatsClient,
    pub metrics: Arc<PublisherMetrics>,
    pub system: Arc<RwLock<System>>,
    pub start_time: Instant,
    pub connection_count: Arc<RwLock<u32>>,
    pub streams: Arc<Streams>,
}

impl SharedState {
    pub async fn new(
        fuel_service: Arc<FuelService>,
        nats_url: &str,
        metrics: Arc<PublisherMetrics>,
        system: Arc<RwLock<System>>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let streams = Streams::new(&nats_client).await;
        Ok(Self {
            fuel_service,
            nats_client,
            metrics,
            system,
            start_time: Instant::now(),
            connection_count: Arc::new(RwLock::new(0)),
            streams: Arc::new(streams),
        })
    }
}

impl SharedState {
    pub fn is_healthy(&self) -> bool {
        if !self.fuel_service.state().started() {
            return false;
        }
        if !self.nats_client.is_connected() {
            return false;
        }
        true
    }

    pub async fn get_health(&self) -> HealthResponse {
        let streams_info = self
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

    pub fn metrics(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();

        // fetch all measured metrics
        let mut buffer = Vec::new();
        if let Err(e) =
            encoder.encode(&self.metrics.registry.gather(), &mut buffer)
        {
            tracing::error!("could not encode custom metrics: {}", e);
        };
        let mut res = match String::from_utf8(buffer.clone()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    "custom metrics could not be from_utf8'd: {}",
                    e
                );
                String::default()
            }
        };
        buffer.clear();

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
            tracing::error!("could not encode prometheus metrics: {}", e);
        };
        let res_custom = match String::from_utf8(buffer.clone()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    "prometheus metrics could not be from_utf8'd: {}",
                    e
                );
                String::default()
            }
        };
        buffer.clear();

        res.push_str(&res_custom);

        // now fetch and add system metrics
        let system_metrics = match self.system.read().metrics() {
            Ok(m) => {
                let metrics = PromSystemMetrics::from(m);
                let labels: Vec<(&str, &str)> = vec![];
                match serde_prometheus::to_string(&metrics, None, labels) {
                    Ok(m) => m,
                    Err(err) => {
                        tracing::error!(
                            "could not encode system metrics: {:?}",
                            err
                        );
                        String::default()
                    }
                }
            }
            Err(err) => {
                tracing::error!(
                    "prometheus system metrics could not be stringified: {:?}",
                    err
                );
                String::default()
            }
        };
        res.push_str(&system_metrics);

        res
    }
}

use std::sync::Arc;

use fuel_core_bin::FuelService;
use fuel_streams_core::nats::{NatsClient, NatsClientOpts};

use crate::metrics::PublisherMetrics;

#[derive(Clone)]
pub struct SharedState {
    pub fuel_service: Arc<FuelService>,
    pub nats_client: NatsClient,
    pub metrics: Arc<PublisherMetrics>,
}

impl SharedState {
    pub async fn new(
        fuel_service: Arc<FuelService>,
        nats_url: &str,
        metrics: Arc<PublisherMetrics>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        Ok(Self {
            fuel_service,
            nats_client,
            metrics,
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

    pub fn metrics(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();

        let mut buffer = Vec::new();
        if let Err(e) =
            encoder.encode(&self.metrics.registry.gather(), &mut buffer)
        {
            tracing::error!("could not encode custom metrics: {}", e);
        };
        let mut res = match String::from_utf8(buffer.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("custom metrics could not be from_utf8'd: {}", e);
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

        res
    }
}

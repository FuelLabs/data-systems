use async_trait::async_trait;
use fuel_web_utils::telemetry::metrics::TelemetryMetrics;
use prometheus::{register_int_counter_vec, IntCounterVec, Registry};

use crate::block_stats::BlockStats;

#[derive(Clone, Debug)]
pub struct Metrics {
    pub registry: Registry,
    pub consumer_success_throughput: IntCounterVec,
    pub consumer_error_throughput: IntCounterVec,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics::new(None).expect("Failed to create default Metrics")
    }
}

#[async_trait]
impl TelemetryMetrics for Metrics {
    fn registry(&self) -> &Registry {
        &self.registry
    }

    fn metrics(&self) -> Option<Self> {
        Some(self.clone())
    }
}

impl Metrics {
    pub fn new(prefix: Option<String>) -> anyhow::Result<Self> {
        let metric_prefix = prefix
            .clone()
            .map(|p| format!("{}_", p))
            .unwrap_or_default();

        let consumer_success_throughput = register_int_counter_vec!(
            format!("{}consumer_metrics_success_throughput", metric_prefix),
            "A metric for the successful throughput of the consumer",
            &["action_type", "block_height", "packets", "duration"],
        )
        .expect("metric must be created");

        let consumer_error_throughput = register_int_counter_vec!(
            format!("{}consumer_metrics_error_throughput", metric_prefix),
            "A metric for the errored throughput of the consumer",
            &["action_type", "block_height", "error", "duration"],
        )
        .expect("metric must be created");

        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");
        registry.register(Box::new(consumer_success_throughput.clone()))?;
        registry.register(Box::new(consumer_error_throughput.clone()))?;

        Ok(Self {
            registry,
            consumer_success_throughput,
            consumer_error_throughput,
        })
    }

    pub fn update_from_stats(&self, stats: &BlockStats) {
        let action_type = stats.action_type.to_string();
        let block_height: u32 = stats.block_height.clone().into();
        let duration = stats.duration_millis();
        match &stats.error {
            Some(error) => {
                // error
                let err = error.to_string();
                self.consumer_error_throughput
                    .with_label_values(&[
                        &action_type,
                        &block_height.to_string(),
                        &err,
                        &duration.to_string(),
                    ])
                    .inc();
            }
            None => {
                // success
                let packets = stats.packet_count;
                self.consumer_success_throughput
                    .with_label_values(&[
                        &action_type,
                        &block_height.to_string(),
                        &packets.to_string(),
                        &duration.to_string(),
                    ])
                    .inc();
            }
        }
    }
}

use async_trait::async_trait;
use fuel_web_utils::telemetry::metrics::TelemetryMetrics;
use prometheus::{
    register_histogram_vec,
    register_int_counter_vec,
    register_int_gauge,
    HistogramVec,
    IntCounterVec,
    IntGauge,
    Registry,
};

use crate::block_stats::BlockStats;

#[derive(Clone, Debug)]
pub struct Metrics {
    pub registry: Registry,
    // Histogram for packet counts
    pub consumer_packets: HistogramVec,
    // Histogram for processing duration
    pub consumer_duration: HistogramVec,
    // Counter for errors,
    pub consumer_error_throughput: IntCounterVec,
    // Gauge for active tasks
    pub consumer_active_tasks: IntGauge,
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
    pub fn new_with_random_prefix() -> anyhow::Result<Self> {
        Metrics::new(Some(Metrics::generate_random_prefix()))
    }

    pub fn new(prefix: Option<String>) -> anyhow::Result<Self> {
        let metric_prefix = prefix
            .clone()
            .map(|p| format!("{}_", p))
            .unwrap_or_default();

        let packet_buckets =
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 100.0, 500.0, 1000.0];
        let duration_buckets = vec![
            1.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0,
        ];

        let consumer_packets = register_histogram_vec!(
            format!("{}consumer_metrics_packets", metric_prefix),
            "Histogram of packet counts processed by the consumer",
            &["action_type"],
            packet_buckets
        )?;

        let consumer_duration = register_histogram_vec!(
            format!("{}consumer_metrics_duration_milliseconds", metric_prefix),
            "Histogram of processing duration in milliseconds",
            &["action_type"],
            duration_buckets
        )?;

        let consumer_error_throughput = register_int_counter_vec!(
            format!("{}consumer_metrics_error_throughput", metric_prefix),
            "A metric for the errored throughput of the consumer",
            &["action_type", "error"],
        )
        .expect("metric must be created");

        let consumer_active_tasks = register_int_gauge!(
            format!("{}consumer_metrics_active_tasks", metric_prefix),
            "Number of currently active processing tasks"
        )?;

        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");
        registry.register(Box::new(consumer_packets.clone()))?;
        registry.register(Box::new(consumer_duration.clone()))?;
        registry.register(Box::new(consumer_error_throughput.clone()))?;
        registry.register(Box::new(consumer_active_tasks.clone()))?;

        Ok(Self {
            registry,
            consumer_packets,
            consumer_duration,
            consumer_error_throughput,
            consumer_active_tasks,
        })
    }

    pub fn update_from_stats(&self, stats: &BlockStats) {
        let action_type = stats.action_type.to_string();
        match &stats.error {
            Some(error) => {
                // error
                let err = error.to_string();
                self.consumer_error_throughput
                    .with_label_values(&[&action_type, &err])
                    .inc();
            }
            None => {
                // success
                let packets = stats.packet_count;
                self.consumer_packets
                    .with_label_values(&[&action_type])
                    .observe(packets as f64);
                self.consumer_duration
                    .with_label_values(&[&action_type])
                    .observe(stats.duration_millis() as f64);
            }
        }
    }

    pub fn set_active_tasks(&self, count: i64) {
        self.consumer_active_tasks.set(count);
    }

    pub fn get_active_tasks(&self) -> i64 {
        self.consumer_active_tasks.get()
    }
}

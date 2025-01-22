use async_trait::async_trait;
use fuel_web_utils::telemetry::metrics::TelemetryMetrics;
use prometheus::{
    register_histogram_vec,
    register_int_counter_vec,
    HistogramVec,
    IntCounterVec,
    Registry,
};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub registry: Registry,
    pub total_published_messages: IntCounterVec,
    pub total_failed_messages: IntCounterVec,
    pub published_messages_throughput: IntCounterVec,
    pub message_size_histogram: HistogramVec,
    pub error_rates: IntCounterVec,
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

        let total_published_messages = register_int_counter_vec!(
            format!(
                "{}publisher_metrics_total_published_messages",
                metric_prefix
            ),
            "A metric counting the number of published messages",
            &[],
        )
        .expect("metric must be created");

        let total_failed_messages = register_int_counter_vec!(
            format!("{}publisher_metrics_total_failed_messages", metric_prefix),
            "A metric counting the number of unpublished and failed messages",
            &[],
        )
        .expect("metric must be created");

        let published_messages_throughput = register_int_counter_vec!(
            format!(
                "{}publisher_metrics_published_messages_throughput",
                metric_prefix
            ),
            "A metric counting the number of published messages per subject",
            &["subject"],
        )
        .expect("metric must be created");

        let message_size_histogram = register_histogram_vec!(
            format!("{}publisher_metrics_message_size_bytes", metric_prefix),
            "Histogram of message sizes in bytes",
            &["subject"],
            vec![100.0, 500.0, 1000.0, 5000.0, 10000.0, 100000.0, 1000000.0]
        )
        .expect("metric must be created");

        let error_rates = register_int_counter_vec!(
            format!("{}publisher_metrics_error_rates", metric_prefix),
            "A metric counting errors or failures during message processing",
            &["subject", "error_type"],
        )
        .expect("metric must be created");

        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");
        registry.register(Box::new(total_published_messages.clone()))?;
        registry.register(Box::new(total_failed_messages.clone()))?;
        registry.register(Box::new(published_messages_throughput.clone()))?;
        registry.register(Box::new(message_size_histogram.clone()))?;
        registry.register(Box::new(error_rates.clone()))?;

        Ok(Self {
            registry,
            total_published_messages,
            total_failed_messages,
            published_messages_throughput,
            message_size_histogram,
            error_rates,
        })
    }

    pub fn update_publisher_success_metrics(
        &self,
        subject: &str,
        published_data_size: usize,
    ) {
        // Update message size histogram
        self.message_size_histogram
            .with_label_values(&[subject])
            .observe(published_data_size as f64);

        // Increment total published messages
        self.total_published_messages.with_label_values(&[]).inc();

        // Increment throughput for the published messages
        self.published_messages_throughput
            .with_label_values(&[subject])
            .inc();
    }

    pub fn update_publisher_error_metrics(&self, subject: &str, error: &str) {
        self.total_failed_messages.with_label_values(&[]).inc();
        self.error_rates.with_label_values(&[subject, error]).inc();
    }
}

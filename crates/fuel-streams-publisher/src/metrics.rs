use prometheus::{
    register_histogram_vec,
    register_int_counter_vec,
    register_int_gauge_vec,
    HistogramVec,
    IntCounterVec,
    IntGaugeVec,
    Registry,
};

#[derive(Clone, Debug)]
pub struct PublisherMetrics {
    pub registry: Registry,
    pub total_subs: IntGaugeVec,
    pub total_published_messages: IntCounterVec,
    pub total_failed_messages: IntCounterVec,
    pub last_published_block_height: IntGaugeVec,
    pub last_published_block_timestamp: IntGaugeVec,
    pub published_messages_throughput: IntCounterVec,
    pub publishing_latency_histogram: HistogramVec,
    pub message_size_histogram: HistogramVec,
    pub error_rates: IntCounterVec,
}

impl Default for PublisherMetrics {
    fn default() -> Self {
        PublisherMetrics::new()
            .expect("Failed to create default PublisherMetrics")
    }
}

impl PublisherMetrics {
    pub fn new() -> anyhow::Result<Self> {
        let total_subs = register_int_gauge_vec!(
            "publisher_metrics_total_subscriptions",
            "A metric counting the number of active subscriptions",
            &["chain_id"],
        )
        .expect("metric can be created");

        let total_published_messages = register_int_counter_vec!(
            "publisher_metrics_total_published_messages",
            "A metric counting the number of published messages",
            &["chain_id", "block_producer"],
        )
        .expect("metric can be created");

        let total_failed_messages = register_int_counter_vec!(
            "publisher_metrics_total_failed_messages",
            "A metric counting the number of unpublished and failed messages",
            &["chain_id", "block_producer"],
        )
        .expect("metric can be created");

        let last_published_block_height = register_int_gauge_vec!(
            "publisher_metrics_last_published_block_height",
            "A metric that represents the last published block height",
            &["chain_id", "block_producer"],
        )
        .expect("metric can be created");

        let last_published_block_timestamp = register_int_gauge_vec!(
            "publisher_metrics_last_published_block_timestamp",
            "A metric that represents the last published transaction timestamp",
            &["chain_id", "block_producer"],
        )
        .expect("metric can be created");

        let published_messages_throughput = register_int_counter_vec!(
            "publisher_metrics_published_messages_throughput",
            "A metric counting the number of published messages per subject wildcard",
            &["chain_id", "block_producer", "subject_wildcard"],
        )
        .expect("metric can be created");

        // New histogram metric for block latency
        let publishing_latency_histogram = register_histogram_vec!(
            "publisher_metrics_block_latency_seconds",
            "Histogram of latencies between receiving and publishing a block",
            &["chain_id", "block_producer", "subject_wildcard"],
            // buckets for latency measurement (e.g., 0.1s, 0.5s, 1s, 5s, 10s)
            vec![0.1, 0.5, 1.0, 5.0, 10.0],
        )
        .expect("metric can be created");

        let message_size_histogram = register_histogram_vec!(
            "publisher_metrics_message_size_bytes",
            "Histogram of message sizes in bytes",
            &["chain_id", "block_producer", "subject_wildcard"],
            vec![100.0, 500.0, 1000.0, 5000.0, 10000.0, 100000.0, 1000000.0]
        )
        .expect("metric can be created");

        let error_rates =
            register_int_counter_vec!(
            "publisher_metrics_error_rates",
            "A metric counting errors or failures during message processing",
            &["chain_id", "block_producer", "subject_wildcard", "error_type"],
        )
            .expect("metric can be created");

        let registry: Registry = Registry::new();
        registry.register(Box::new(total_subs.clone()))?;
        registry.register(Box::new(total_published_messages.clone()))?;
        registry.register(Box::new(total_failed_messages.clone()))?;
        registry.register(Box::new(last_published_block_height.clone()))?;
        registry.register(Box::new(last_published_block_timestamp.clone()))?;
        registry.register(Box::new(published_messages_throughput.clone()))?;
        registry.register(Box::new(publishing_latency_histogram.clone()))?;
        registry.register(Box::new(message_size_histogram.clone()))?;
        registry.register(Box::new(error_rates.clone()))?;

        Ok(Self {
            registry,
            total_subs,
            total_published_messages,
            total_failed_messages,
            last_published_block_height,
            last_published_block_timestamp,
            published_messages_throughput,
            publishing_latency_histogram,
            message_size_histogram,
            error_rates,
        })
    }
}

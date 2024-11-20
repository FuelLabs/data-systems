use std::sync::Arc;

use chrono::Utc;
use fuel_core::database::database_description::DatabaseHeight;
use fuel_streams_core::prelude::*;
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
        PublisherMetrics::new(None)
            .expect("Failed to create default PublisherMetrics")
    }
}

impl PublisherMetrics {
    pub fn new(prefix: Option<String>) -> anyhow::Result<Self> {
        let metric_prefix = prefix
            .clone()
            .map(|p| format!("{}_", p))
            .unwrap_or_default();

        let total_subs = register_int_gauge_vec!(
            format!("{}publisher_metrics_total_subscriptions", metric_prefix),
            "A metric counting the number of active subscriptions",
            &["chain_id"],
        )
        .expect("metric must be created");

        let total_published_messages = register_int_counter_vec!(
            format!(
                "{}publisher_metrics_total_published_messages",
                metric_prefix
            ),
            "A metric counting the number of published messages",
            &["chain_id", "block_producer"],
        )
        .expect("metric must be created");

        let total_failed_messages = register_int_counter_vec!(
            format!("{}publisher_metrics_total_failed_messages", metric_prefix),
            "A metric counting the number of unpublished and failed messages",
            &["chain_id", "block_producer"],
        )
        .expect("metric must be created");

        let last_published_block_height = register_int_gauge_vec!(
            format!(
                "{}publisher_metrics_last_published_block_height",
                metric_prefix
            ),
            "A metric that represents the last published block height",
            &["chain_id", "block_producer"],
        )
        .expect("metric must be created");

        let last_published_block_timestamp = register_int_gauge_vec!(
            format!(
                "{}publisher_metrics_last_published_block_timestamp",
                metric_prefix
            ),
            "A metric that represents the last published transaction timestamp",
            &["chain_id", "block_producer"],
        )
        .expect("metric must be created");

        let published_messages_throughput = register_int_counter_vec!(
            format!("{}publisher_metrics_published_messages_throughput", metric_prefix),
            "A metric counting the number of published messages per subject wildcard",
            &["chain_id", "block_producer", "subject_wildcard"],
        )
        .expect("metric must be created");

        // New histogram metric for block latency
        let publishing_latency_histogram = register_histogram_vec!(
            format!("{}publisher_metrics_block_latency_seconds", metric_prefix),
            "Histogram of latencies between receiving and publishing a block",
            &["chain_id", "block_producer", "subject_wildcard"],
            // buckets for latency measurement (e.g., 0.1s, 0.5s, 1s, 5s, 10s)
            vec![0.1, 0.5, 1.0, 5.0, 10.0],
        )
        .expect("metric must be created");

        let message_size_histogram = register_histogram_vec!(
            format!("{}publisher_metrics_message_size_bytes", metric_prefix),
            "Histogram of message sizes in bytes",
            &["chain_id", "block_producer", "subject_wildcard"],
            vec![100.0, 500.0, 1000.0, 5000.0, 10000.0, 100000.0, 1000000.0]
        )
        .expect("metric must be created");

        let error_rates =
            register_int_counter_vec!(
            format!("{}publisher_metrics_error_rates", metric_prefix),
            "A metric counting errors or failures during message processing",
            &["chain_id", "block_producer", "subject_wildcard", "error_type"],
        )
            .expect("metric must be created");

        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");
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

#[allow(dead_code)]
// TODO: Will this be useful in the future?
pub fn add_block_metrics(
    chain_id: &ChainId,
    block: &FuelCoreBlock<Transaction>,
    block_producer: &Address,
    metrics: &Arc<PublisherMetrics>,
) -> anyhow::Result<Arc<PublisherMetrics>> {
    let latency = Utc::now().timestamp() - block.header().time().to_unix();

    metrics
        .publishing_latency_histogram
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
            BlocksSubject::WILDCARD,
        ])
        .observe(latency as f64);

    metrics
        .last_published_block_timestamp
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
        ])
        .set(block.header().time().to_unix());

    metrics
        .last_published_block_height
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
        ])
        .set(block.header().consensus().height.as_u64() as i64);

    Ok(metrics.to_owned())
}

#[cfg(test)]
mod tests {
    use prometheus::{gather, Encoder, TextEncoder};

    use super::*;

    impl PublisherMetrics {
        pub fn random() -> Self {
            use rand::{distributions::Alphanumeric, Rng};

            let prefix = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .filter(|c| c.is_ascii_alphabetic())
                .take(6)
                .map(char::from)
                .collect();

            PublisherMetrics::new(Some(prefix))
                .expect("Failed to create random PublisherMetrics")
        }
    }

    #[test]
    fn test_total_published_messages_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .total_published_messages
            .with_label_values(&["chain_id_1", "block_producer_1"])
            .inc_by(5);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("publisher_metrics_total_published_messages"));
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("5"));
    }

    #[test]
    fn test_latency_histogram_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .publishing_latency_histogram
            .with_label_values(&["chain_id_1", "block_producer_1", "topic_1"])
            .observe(0.75);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("publisher_metrics_block_latency_seconds"));
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("topic_1"));
        assert!(output.contains("0.75"));
    }

    #[test]
    fn test_message_size_histogram_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .message_size_histogram
            .with_label_values(&["chain_id_1", "block_producer_1", "topic_1"])
            .observe(1500.1);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("publisher_metrics_message_size_bytes"));
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("topic_1"));
        assert!(output.contains("1500.1"));
    }

    #[test]
    fn test_total_failed_messages_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .total_failed_messages
            .with_label_values(&["chain_id_1", "block_producer_1"])
            .inc_by(3);

        // Gather all the metrics
        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        // Convert the gathered output to a string
        let output = String::from_utf8(buffer.clone()).unwrap();

        // Assert that the output contains the correct failed message metric
        assert!(output.contains("publisher_metrics_total_failed_messages"));
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("3"));
    }

    #[test]
    fn test_total_subs_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .total_subs
            .with_label_values(&["chain_id_1"])
            .set(10);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("publisher_metrics_total_subscriptions"));
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_last_published_block_height_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .last_published_block_height
            .with_label_values(&["chain_id_1", "block_producer_1"])
            .set(1234);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(
            output.contains("publisher_metrics_last_published_block_height")
        );
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("1234"));
    }

    #[test]
    fn test_last_published_block_timestamp_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .last_published_block_timestamp
            .with_label_values(&["chain_id_1", "block_producer_1"])
            .set(1633046400);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(
            output.contains("publisher_metrics_last_published_block_timestamp")
        );
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("1633046400"));
    }

    #[test]
    fn test_published_messages_throughput_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .published_messages_throughput
            .with_label_values(&["chain_id_1", "block_producer_1", "topic_1"])
            .inc_by(10);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(
            output.contains("publisher_metrics_published_messages_throughput")
        );
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("topic_1"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_error_rates_metric() {
        let metrics = PublisherMetrics::random();

        metrics
            .error_rates
            .with_label_values(&[
                "chain_id_1",
                "block_producer_1",
                "topic_1",
                "timeout",
            ])
            .inc_by(1);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("publisher_metrics_error_rates"));
        assert!(output.contains("chain_id_1"));
        assert!(output.contains("block_producer_1"));
        assert!(output.contains("topic_1"));
        assert!(output.contains("timeout"));
        assert!(output.contains("1"));
    }
}

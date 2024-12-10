use prometheus::{
    register_int_counter_vec,
    register_int_gauge_vec,
    IntCounterVec,
    IntGaugeVec,
    Registry,
};
use rand::{distributions::Alphanumeric, Rng};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub registry: Registry,
    pub total_ws_subs: IntGaugeVec,
    pub user_subscribed_messages: IntCounterVec,
    pub subs_messages_throughput: IntCounterVec,
    pub subs_messages_error_rates: IntCounterVec,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics::new(None).expect("Failed to create default Metrics")
    }
}

impl Metrics {
    pub fn generate_random_prefix() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| c.is_ascii_alphabetic())
            .take(6)
            .map(char::from)
            .collect()
    }

    pub fn new_with_random_prefix() -> anyhow::Result<Self> {
        Metrics::new(Some(Metrics::generate_random_prefix()))
    }

    pub fn new(prefix: Option<String>) -> anyhow::Result<Self> {
        let metric_prefix = prefix
            .clone()
            .map(|p| format!("{}_", p))
            .unwrap_or_default();

        let total_ws_subs = register_int_gauge_vec!(
            format!("{}ws_streamer_metrics_total_subscriptions", metric_prefix),
            "A metric counting the number of active ws subscriptions",
            &[],
        )
        .expect("metric must be created");

        let user_subscribed_messages = register_int_counter_vec!(
            format!(
                "{}ws_streamer_metrics_user_subscribed_messages",
                metric_prefix
            ),
            "A metric counting the number of published messages",
            &["user_id", "subject_wildcard"],
        )
        .expect("metric must be created");

        let subs_messages_throughput = register_int_counter_vec!(
            format!("{}ws_streamer_metrics_subs_messages_throughput", metric_prefix),
            "A metric counting the number of subscription messages per subject wildcard",
            &["subject_wildcard"],
        )
        .expect("metric must be created");

        let subs_messages_error_rates =
            register_int_counter_vec!(
            format!("{}ws_streamer_metrics_subs_messages_error_rates", metric_prefix),
            "A metric counting errors or failures during subscription message processing",
            &["subject_wildcard", "error_type"],
        )
            .expect("metric must be created");

        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");
        registry.register(Box::new(total_ws_subs.clone()))?;
        registry.register(Box::new(user_subscribed_messages.clone()))?;
        registry.register(Box::new(subs_messages_throughput.clone()))?;
        registry.register(Box::new(subs_messages_error_rates.clone()))?;

        Ok(Self {
            registry,
            total_ws_subs,
            user_subscribed_messages,
            subs_messages_throughput,
            subs_messages_error_rates,
        })
    }
}

#[cfg(test)]
mod tests {
    use prometheus::{gather, Encoder, TextEncoder};

    use super::*;

    impl Metrics {
        pub fn random() -> Self {
            Metrics::new_with_random_prefix()
                .expect("Failed to create random Metrics")
        }
    }

    #[test]
    fn test_user_subscribed_messages_metric() {
        let metrics = Metrics::random();

        metrics
            .user_subscribed_messages
            .with_label_values(&["user_id_1", "subject_wildcard_1"])
            .inc_by(5);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("ws_streamer_metrics_user_subscribed_messages"));
        assert!(output.contains("user_id_1"));
        assert!(output.contains("subject_wildcard_1"));
        assert!(output.contains("5"));
    }

    #[test]
    fn test_subs_messages_total_metric() {
        let metrics = Metrics::random();

        metrics.total_ws_subs.with_label_values(&[]).set(10);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("ws_streamer_metrics_total_subscriptions"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_subs_messages_throughput_metric() {
        let metrics = Metrics::random();

        metrics
            .subs_messages_throughput
            .with_label_values(&["wildcard_1"])
            .inc_by(10);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("ws_streamer_metrics_subs_messages_throughput"));
        assert!(output.contains("wildcard_1"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_subs_messages_error_rates_metric() {
        let metrics = Metrics::random();

        metrics
            .subs_messages_error_rates
            .with_label_values(&["wildcard_1", "timeout"])
            .inc_by(1);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(
            output.contains("ws_streamer_metrics_subs_messages_error_rates")
        );
        assert!(output.contains("wildcard_1"));
        assert!(output.contains("timeout"));
        assert!(output.contains("1"));
    }
}

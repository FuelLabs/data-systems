use std::time::Duration;

use async_trait::async_trait;
use fuel_web_utils::telemetry::metrics::TelemetryMetrics;
use prometheus::{
    register_histogram,
    register_int_counter,
    register_int_counter_vec,
    register_int_gauge,
    Histogram,
    IntCounter,
    IntCounterVec,
    IntGauge,
    Registry,
};

#[derive(Debug)]
pub enum SubscriptionChange {
    Added,
    Removed,
}

#[derive(Clone, Debug)]
pub struct Metrics {
    pub registry: Registry,
    pub total_ws_subs: IntGauge,
    pub user_subscribed_messages: IntGauge,
    pub subs_messages_throughput: IntCounter,
    pub subs_messages_error_rates: IntCounterVec,
    pub connection_duration: Histogram,
    pub duplicate_subscription_attempts: IntCounter,
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

        let total_ws_subs = register_int_gauge!(
            format!("{}ws_streamer_metrics_total_subscriptions", metric_prefix),
            "A metric counting the number of active ws subscriptions",
        )
        .expect("metric must be created");

        let user_subscribed_messages = register_int_gauge!(
            format!(
                "{}ws_streamer_metrics_user_subscribed_messages",
                metric_prefix
            ),
            "A metric counting the number of published messages",
        )
        .expect("metric must be created");

        let subs_messages_throughput = register_int_counter!(
            format!(
                "{}ws_streamer_metrics_subs_messages_throughput",
                metric_prefix
            ),
            "A metric counting the number of subscription messages",
        )
        .expect("metric must be created");

        let subs_messages_error_rates =
            register_int_counter_vec!(
            format!("{}ws_streamer_metrics_subs_messages_error_rates", metric_prefix),
            "A metric counting errors or failures during subscription message processing",
            &["error_type"],
        )
            .expect("metric must be created");

        let connection_duration = register_histogram!(
            format!("{}ws_connection_duration_seconds", metric_prefix),
            "Duration of WebSocket connections in seconds",
            vec![0.1, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]
        )
        .expect("metric must be created");

        let duplicate_subscription_attempts = register_int_counter!(
            format!("{}ws_duplicate_subscription_attempts", metric_prefix),
            "Number of attempts to create duplicate subscriptions",
        )
        .expect("metric must be created");

        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");
        registry.register(Box::new(total_ws_subs.clone()))?;
        registry.register(Box::new(user_subscribed_messages.clone()))?;
        registry.register(Box::new(subs_messages_throughput.clone()))?;
        registry.register(Box::new(subs_messages_error_rates.clone()))?;
        registry.register(Box::new(connection_duration.clone()))?;
        registry.register(Box::new(duplicate_subscription_attempts.clone()))?;

        Ok(Self {
            registry,
            total_ws_subs,
            user_subscribed_messages,
            subs_messages_throughput,
            subs_messages_error_rates,
            connection_duration,
            duplicate_subscription_attempts,
        })
    }

    pub fn update_user_subscription_metrics(&self) {
        self.user_subscribed_messages.inc();

        // Increment throughput for the subscribed messages
        self.subs_messages_throughput.inc();
    }

    pub fn update_error_metrics(&self, subject: &str, error_type: &str) {
        self.subs_messages_error_rates
            .with_label_values(&[subject, error_type])
            .inc();
    }

    pub fn increment_subscriptions_count(&self) {
        self.total_ws_subs.inc();
    }

    pub fn decrement_subscriptions_count(&self) {
        self.total_ws_subs.dec();
    }

    pub fn update_unsubscribed(&self) {
        self.user_subscribed_messages.dec();
    }

    pub fn update_subscribed(&self) {
        self.user_subscribed_messages.inc();
    }

    pub fn track_connection_duration(&self, duration: Duration) {
        self.connection_duration.observe(duration.as_secs_f64());
    }

    pub fn track_duplicate_subscription(&self) {
        self.duplicate_subscription_attempts.inc();
    }

    pub fn update_user_subscription_count(&self, change: &SubscriptionChange) {
        let delta = match change {
            SubscriptionChange::Added => 1,
            SubscriptionChange::Removed => -1,
        };

        // Update subject-specific count
        self.user_subscribed_messages.add(delta);
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

        metrics.user_subscribed_messages.set(5);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("ws_streamer_metrics_user_subscribed_messages"));
        assert!(output.contains("5"));
    }

    #[test]
    fn test_subs_messages_total_metric() {
        let metrics = Metrics::random();

        metrics.total_ws_subs.set(10);

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

        metrics.subs_messages_throughput.inc_by(10);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("ws_streamer_metrics_subs_messages_throughput"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_subs_messages_error_rates_metric() {
        let metrics = Metrics::random();

        metrics
            .subs_messages_error_rates
            .with_label_values(&["timeout"])
            .inc_by(1);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(
            output.contains("ws_streamer_metrics_subs_messages_error_rates")
        );
        assert!(output.contains("timeout"));
        assert!(output.contains("1"));
    }
}

use std::time::Duration;

use async_trait::async_trait;
use fuel_web_utils::{api_key::ApiKeyId, telemetry::metrics::TelemetryMetrics};
use prometheus::{
    register_gauge,
    register_histogram_vec,
    register_int_counter_vec,
    Gauge,
    HistogramVec,
    IntCounterVec,
    Registry,
};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub registry: Registry,
    pub db_queries_error_rates: IntCounterVec,
    pub connection_duration: HistogramVec,
    pub latest_block_height: Gauge,
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

        let db_queries_error_rates = register_int_counter_vec!(
            format!("{}api_db_queries_error_rates", metric_prefix),
            "A metric counting errors or failures during queries execution",
            &["subject", "error_type"],
        )
        .expect("metric must be created");

        let connection_duration = register_histogram_vec!(
            format!("{}api_duration_seconds", metric_prefix),
            "Duration of connections in seconds",
            &["user_id", "user_name"],
            vec![0.1, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]
        )
        .expect("metric must be created");

        let latest_block_height = register_gauge!(
            format!("{}api_latest_block_height", metric_prefix),
            "The latest block height in the database"
        )
        .expect("metric must be created");

        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");
        registry.register(Box::new(db_queries_error_rates.clone()))?;
        registry.register(Box::new(connection_duration.clone()))?;

        Ok(Self {
            registry,
            db_queries_error_rates,
            connection_duration,
            latest_block_height,
        })
    }

    pub fn update_query_error_metrics(&self, query: &str, error_type: &str) {
        self.db_queries_error_rates
            .with_label_values(&[query, error_type])
            .inc();
    }

    pub fn track_connection_duration(
        &self,
        user_id: ApiKeyId,
        user_name: &str,
        duration: Duration,
    ) {
        self.connection_duration
            .with_label_values(&[&user_id.to_string(), user_name])
            .observe(duration.as_secs_f64());
    }

    pub fn update_latest_block_height(&self, height: u32) {
        self.latest_block_height.set(height as f64);
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
    fn test_db_queries_error_rates_metric() {
        let metrics = Metrics::random();

        metrics
            .db_queries_error_rates
            .with_label_values(&["query_1", "timeout"])
            .inc_by(1);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("api_db_queries_error_rates"));
        assert!(output.contains("query_1"));
        assert!(output.contains("timeout"));
        assert!(output.contains("1"));
    }

    #[test]
    fn test_latest_block_height_metric() {
        let metrics = Metrics::random();

        metrics.update_latest_block_height(123);

        let metric_families = gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.clone()).unwrap();

        assert!(output.contains("api_latest_block_height"));
        assert!(output.contains("123"));
    }
}

use async_trait::async_trait;
use prometheus::Registry;
use rand::{distr::Alphanumeric, Rng};

#[async_trait]
pub trait TelemetryMetrics: Send + Sync + 'static {
    fn registry(&self) -> &Registry;

    fn metrics(&self) -> Option<Self>
    where
        Self: std::marker::Sized;

    fn generate_random_prefix() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| c.is_ascii_alphabetic())
            .take(6)
            .map(char::from)
            .collect()
    }

    fn gather_metrics(&self) -> String {
        use prometheus::Encoder;

        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();

        if let Err(e) = encoder.encode(&self.registry().gather(), &mut buffer) {
            tracing::error!("could not encode custom metrics: {}", e);
        }

        String::from_utf8(buffer).unwrap_or_default()
    }
}

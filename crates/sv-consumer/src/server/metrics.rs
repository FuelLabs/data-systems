use async_trait::async_trait;
use fuel_web_utils::telemetry::metrics::TelemetryMetrics;
use prometheus::Registry;

#[derive(Clone, Debug)]
pub struct Metrics {
    // TODO: add more metrics
    pub registry: Registry,
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
        let registry =
            Registry::new_custom(prefix, None).expect("registry to be created");

        Ok(Self { registry })
    }
}

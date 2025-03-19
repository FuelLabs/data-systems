mod elastic_search;
pub mod metrics;
mod runtime;
#[allow(clippy::needless_borrows_for_generic_args)]
mod system;

use std::sync::Arc;

use anyhow::Context;
use elastic_search::{
    new_elastic_search,
    should_use_elasticsearch,
    ElasticSearch,
    LogEntry,
};
use metrics::TelemetryMetrics;
// TODO: Consider using tokio's Rwlock instead
use runtime::Runtime;

#[derive(Clone)]
pub struct Telemetry<M: TelemetryMetrics> {
    runtime: Arc<Runtime>,
    metrics: Option<Arc<M>>,
    elastic_search: Option<Arc<ElasticSearch>>,
}

impl<M: TelemetryMetrics> Telemetry<M> {
    const DEDICATED_THREADS: usize = 2;

    pub async fn new(metrics: Option<M>) -> anyhow::Result<Arc<Self>> {
        let runtime = Runtime::new(Self::DEDICATED_THREADS);
        let elastic_search = if should_use_elasticsearch() {
            Some(Arc::new(new_elastic_search().await?))
        } else {
            None
        };

        Ok(Arc::new(Self {
            runtime: Arc::new(runtime),
            metrics: if should_use_metrics() {
                metrics.map(Arc::new)
            } else {
                None
            },
            elastic_search,
        }))
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        if let Some(elastic_search) = self.elastic_search.as_ref() {
            tracing::info!(
                "Elastic Search connection live? {:?}",
                elastic_search.get_conn().check_alive().unwrap_or_default()
            );
            elastic_search
                .get_conn()
                .ping()
                .await
                .context("Error pinging elastisearch connection")?;
            tracing::info!("Elastic logger pinged successfully!");
        };

        Ok(())
    }

    pub fn base_metrics(&self) -> Option<M> {
        self.metrics.clone().and_then(|m| m.metrics())
    }

    pub fn log_info(&self, message: &str) {
        let entry = LogEntry::new("INFO", message);
        self.maybe_elog(entry);
        tracing::info!("{}", message);
    }

    pub fn log_error(&self, message: &str) {
        let entry = LogEntry::new("ERROR", message);
        self.maybe_elog(entry);
        tracing::error!("{}", message);
    }

    fn maybe_elog(&self, entry: LogEntry) {
        if let Some(elastic_search) = &self.elastic_search {
            self.runtime
                .spawn(elastic_search::log(elastic_search.clone(), entry));
        }
    }

    pub fn maybe_use_metrics<F>(&self, f: F)
    where
        F: Fn(&M),
    {
        if let Some(metrics) = &self.metrics {
            f(metrics);
        }
    }

    pub async fn get_metrics(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();

        if self.metrics.is_none() {
            return "# EOF\n".to_string();
        }

        let mut result = String::new();
        if let Some(metrics) = &self.metrics {
            result.push_str(&metrics.gather_metrics());
        }

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
            tracing::error!("could not encode prometheus metrics: {}", e);
        }

        let res_custom = match String::from_utf8(buffer) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    "prometheus metrics could not be from_utf8'd: {}",
                    e
                );
                String::default()
            }
        };

        result.push_str(&res_custom);
        result.push_str("# EOF\n");
        result
    }
}

pub fn should_use_metrics() -> bool {
    dotenvy::var("USE_METRICS").is_ok_and(|val| val == "true")
}

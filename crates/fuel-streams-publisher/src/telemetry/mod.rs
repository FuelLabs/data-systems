mod elastic_search;
mod publisher;
mod runtime;
#[allow(clippy::needless_borrows_for_generic_args)]
mod system;

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use elastic_search::{
    new_elastic_search,
    should_use_elasticsearch,
    ElasticSearch,
    LogEntry,
};
use fuel_streams_core::prelude::*;
// TODO: Consider using tokio's Rwlock instead
use parking_lot::RwLock;
use publisher::PublisherMetrics;
use runtime::Runtime;
use system::{System, SystemMetricsWrapper};

#[derive(Clone)]
pub struct Telemetry {
    runtime: Arc<Runtime>,
    system: Arc<RwLock<System>>,
    publisher_metrics: Option<Arc<PublisherMetrics>>,
    elastic_search: Option<Arc<ElasticSearch>>,
}

impl Telemetry {
    const DEDICATED_THREADS: usize = 2;

    pub async fn new() -> anyhow::Result<Arc<Self>> {
        let runtime =
            Runtime::new(Self::DEDICATED_THREADS, Duration::from_secs(20));
        let system = Arc::new(RwLock::new(System::new().await));

        let publisher_metrics = if should_use_publisher_metrics() {
            Some(Arc::new(PublisherMetrics::default()))
        } else {
            None
        };

        let elastic_search = if should_use_elasticsearch() {
            Some(Arc::new(new_elastic_search().await?))
        } else {
            None
        };

        Ok(Arc::new(Self {
            runtime: Arc::new(runtime),
            system,
            publisher_metrics,
            elastic_search,
        }))
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let system = Arc::clone(&self.system);

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

        self.runtime.start(move || {
            system.write().refresh();
        });

        Ok(())
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

    pub fn update_publisher_success_metrics(
        &self,
        subject: &str,
        published_data_size: usize,
        chain_id: &FuelCoreChainId,
        block_producer: &Address,
    ) {
        self.maybe_use_metrics(|metrics| {
            // Update message size histogram
            metrics
                .message_size_histogram
                .with_label_values(&[
                    &chain_id.to_string(),
                    &block_producer.to_string(),
                    subject,
                ])
                .observe(published_data_size as f64);

            // Increment total published messages
            metrics
                .total_published_messages
                .with_label_values(&[
                    &chain_id.to_string(),
                    &block_producer.to_string(),
                ])
                .inc();

            // Increment throughput for the published messages
            metrics
                .published_messages_throughput
                .with_label_values(&[
                    &chain_id.to_string(),
                    &block_producer.to_string(),
                    subject,
                ])
                .inc();
        });
    }

    pub fn update_publisher_error_metrics(
        &self,
        subject: &str,
        chain_id: &FuelCoreChainId,
        block_producer: &Address,
        error: &str,
    ) {
        self.maybe_use_metrics(|metrics| {
            metrics
                .error_rates
                .with_label_values(&[
                    &chain_id.to_string(),
                    &block_producer.to_string(),
                    subject,
                    error,
                ])
                .inc();
        });
    }

    pub fn record_streams_count(
        &self,
        chain_id: &FuelCoreChainId,
        count: usize,
    ) {
        self.maybe_use_metrics(|metrics| {
            metrics
                .total_subs
                .with_label_values(&[&chain_id.to_string()])
                .set(count as i64);
        });
    }

    pub fn record_failed_publishing(
        &self,
        chain_id: &FuelCoreChainId,
        block_producer: &Address,
    ) {
        self.maybe_use_metrics(|metrics| {
            metrics
                .total_failed_messages
                .with_label_values(&[
                    &chain_id.to_string(),
                    &block_producer.to_string(),
                ])
                .inc();
        });
    }

    pub fn maybe_use_metrics<F>(&self, f: F)
    where
        F: Fn(&PublisherMetrics),
    {
        if let Some(metrics) = &self.publisher_metrics {
            f(metrics);
        }
    }

    // TODO: Break into smaller functions
    pub async fn get_metrics(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();

        if self.publisher_metrics.is_none() {
            return "".to_string();
        }

        // fetch all measured metrics
        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(
            &self.publisher_metrics.as_ref().unwrap().registry.gather(),
            &mut buffer,
        ) {
            tracing::error!("could not encode custom metrics: {}", e);
        };
        let mut res = match String::from_utf8(buffer.clone()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    "custom metrics could not be from_utf8'd: {}",
                    e
                );
                String::default()
            }
        };
        buffer.clear();

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
            tracing::error!("could not encode prometheus metrics: {}", e);
        };
        let res_custom = match String::from_utf8(buffer.clone()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    "prometheus metrics could not be from_utf8'd: {}",
                    e
                );
                String::default()
            }
        };
        buffer.clear();

        res.push_str(&res_custom);

        // now fetch and add system metrics
        let system_metrics = match self.system.read().metrics() {
            Ok(m) => {
                let metrics = SystemMetricsWrapper::from(m);
                let labels: Vec<(&str, &str)> = vec![];
                match serde_prometheus::to_string(&metrics, None, labels) {
                    Ok(m) => m,
                    Err(err) => {
                        tracing::error!(
                            "could not encode system metrics: {:?}",
                            err
                        );
                        String::default()
                    }
                }
            }
            Err(err) => {
                tracing::error!(
                    "prometheus system metrics could not be stringified: {:?}",
                    err
                );
                String::default()
            }
        };
        res.push_str(&system_metrics);

        res
    }
}

pub fn should_use_publisher_metrics() -> bool {
    dotenvy::var("USE_METRICS").is_ok_and(|val| val == "true")
}

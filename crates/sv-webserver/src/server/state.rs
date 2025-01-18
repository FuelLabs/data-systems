use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::FuelStreams;
use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::{
    server::{
        middlewares::api_key::{ApiKeysManager, KeyStorage},
        state::StateProvider,
    },
    telemetry::Telemetry,
};

use crate::{config::Config, metrics::Metrics};

#[derive(Clone)]
pub struct ServerState {
    pub start_time: Instant,
    pub msg_broker: Arc<dyn MessageBroker>,
    pub fuel_streams: Arc<FuelStreams>,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub db: Arc<Db>,
    pub api_keys_manager: Arc<ApiKeysManager>,
}

impl ServerState {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let msg_broker =
            MessageBrokerClient::Nats.start(&config.broker.url).await?;

        let db = Db::new(DbConnectionOpts {
            connection_str: config.db.url.clone(),
            ..Default::default()
        })
        .await?
        .arc();

        let fuel_streams = FuelStreams::new(&msg_broker, &db).await.arc();
        let metrics = Metrics::new_with_random_prefix()?;
        let telemetry = Telemetry::new(Some(metrics)).await?;
        telemetry.start().await?;

        let api_keys_manager = Arc::new(ApiKeysManager::new(&db));
        let initial_keys = api_keys_manager.load_from_db().await?;
        for key in initial_keys {
            if let Err(e) = api_keys_manager.storage.insert(&key) {
                tracing::warn!(
                    error = %e,
                    "Failed to cache initial API key"
                );
            }
        }

        Ok(Self {
            db,
            start_time: Instant::now(),
            msg_broker,
            fuel_streams,
            telemetry,
            api_keys_manager,
        })
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[async_trait]
impl StateProvider for ServerState {
    async fn is_healthy(&self) -> bool {
        self.msg_broker.is_healthy().await
    }

    async fn get_health(&self) -> serde_json::Value {
        self.msg_broker
            .get_health_info(self.uptime().as_secs())
            .await
            .unwrap_or(serde_json::json!({}))
    }

    async fn get_metrics(&self) -> String {
        self.telemetry.get_metrics().await
    }
}

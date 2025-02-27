use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::FuelStreams;
use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::{
    api_key::{ApiKeysManager, KeyStorage},
    server::{middlewares::password::PasswordManager, state::StateProvider},
    telemetry::Telemetry,
};

use crate::{config::Config, metrics::Metrics, API_PASSWORD};

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<Db>,
    pub start_time: Instant,
    pub msg_broker: Arc<NatsMessageBroker>,
    pub fuel_streams: Arc<FuelStreams>,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub api_keys_manager: Arc<ApiKeysManager>,
    pub password_manager: Arc<PasswordManager>,
}

impl ServerState {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let url = &config.broker.url;
        let msg_broker = NatsMessageBroker::setup(url, None).await?;
        let db = Db::new(DbConnectionOpts {
            connection_str: config.db.url.clone(),
            ..Default::default()
        })
        .await?;

        let fuel_streams = FuelStreams::new(&msg_broker, &db).await.arc();
        let metrics = Metrics::new(None)?;
        let telemetry = Telemetry::new(Some(metrics)).await?;
        telemetry.start().await?;

        let api_keys_manager = Arc::new(ApiKeysManager::new());
        let initial_keys = api_keys_manager.load_from_db(&db).await?;
        for key in initial_keys {
            if let Err(e) = api_keys_manager.storage().insert(&key) {
                tracing::warn!(
                    error = %e,
                    "Failed to cache initial API key"
                );
            }
        }

        let password_manager =
            Arc::new(PasswordManager::new(API_PASSWORD.clone()));

        Ok(Self {
            db,
            start_time: Instant::now(),
            msg_broker,
            fuel_streams,
            telemetry,
            api_keys_manager,
            password_manager,
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

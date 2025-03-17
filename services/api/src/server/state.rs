use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::{
    api_key::{ApiKeysManager, KeyStorage},
    server::state::StateProvider,
    telemetry::Telemetry,
};

use crate::{config::Config, metrics::Metrics};

#[derive(Clone)]
pub struct ServerState {
    pub start_time: Instant,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub db: Arc<Db>,
    pub api_keys_manager: Arc<ApiKeysManager>,
}

impl ServerState {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let db = Db::new(DbConnectionOpts {
            connection_str: config.db.url.clone(),
            ..Default::default()
        })
        .await?;
        tracing::info!("Connected to database at {}", config.db.url);

        let metrics = Metrics::new(None)?;
        let telemetry = Telemetry::new(Some(metrics)).await?;
        telemetry.start().await?;
        tracing::info!("Initialized telemetry");

        let api_keys_manager = Arc::new(ApiKeysManager::default());
        let initial_keys = api_keys_manager.load_from_db(&db).await?;
        for key in initial_keys {
            if let Err(e) = api_keys_manager.storage().insert(&key) {
                tracing::warn!(
                    error = %e,
                    "Failed to cache initial API key"
                );
            }
        }
        tracing::info!("Initialized api key manager");

        Ok(Self {
            db,
            start_time: Instant::now(),
            telemetry,
            api_keys_manager,
        })
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    async fn check_db(&self) -> Result<bool, sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.db.pool)
            .await
            .map(|_| true)
    }
}

#[async_trait]
impl StateProvider for ServerState {
    async fn is_healthy(&self) -> bool {
        self.check_db().await.is_ok()
    }

    async fn get_health(&self) -> serde_json::Value {
        serde_json::json!({
            "uptime": self.uptime().as_secs(),
        })
    }

    async fn get_metrics(&self) -> String {
        self.telemetry.get_metrics().await
    }
}

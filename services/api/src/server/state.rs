use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use fuel_streams_domains::infra::{Db, DbConnectionOpts};
use fuel_web_utils::{
    api_key::ApiKeysManager,
    server::state::StateProvider,
    telemetry::Telemetry,
};

use crate::{config::Config, metrics::Metrics};

#[derive(Clone)]
pub struct ServerState {
    pub start_time: Instant,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub db: Arc<Db>,
    pub db_write: Arc<Db>,
    pub api_keys_manager: Arc<ApiKeysManager>,
}

impl ServerState {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let db = Db::new(DbConnectionOpts {
            connection_str: config.db.read_url.clone(),
            ..Default::default()
        })
        .await?;
        tracing::info!("Connected to read database at {}", config.db.read_url);
        let db_write = Db::new(DbConnectionOpts {
            connection_str: config.db.url.clone(),
            min_connections: Some(1),
            pool_size: Some(1),
            ..Default::default()
        })
        .await?;
        tracing::info!("Connected to write database at {}", config.db.url);

        let metrics = Metrics::new(None)?;
        let telemetry = Telemetry::new(Some(metrics)).await?;
        telemetry.start().await?;
        tracing::info!("Initialized telemetry");

        let api_keys_manager = Arc::new(ApiKeysManager::default());

        Ok(Self {
            db,
            db_write,
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

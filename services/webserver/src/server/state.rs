use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::FuelStreams;
use fuel_streams_domains::infra::{Db, DbConnectionOpts};
use fuel_web_utils::{
    api_key::ApiKeysManager,
    server::state::StateProvider,
    telemetry::Telemetry,
};

use crate::{
    config::Config,
    metrics::Metrics,
    server::websocket::ConnectionChecker,
};

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<Db>,
    pub start_time: Instant,
    pub msg_broker: Arc<NatsMessageBroker>,
    pub fuel_streams: Arc<FuelStreams>,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub api_keys_manager: Arc<ApiKeysManager>,
    pub connection_checker: Arc<ConnectionChecker>,
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
        let api_keys_manager = Arc::new(ApiKeysManager::default());

        let connection_checker = Arc::new(ConnectionChecker::default());
        connection_checker.start().await;

        Ok(Self {
            db,
            start_time: Instant::now(),
            msg_broker,
            fuel_streams,
            telemetry,
            api_keys_manager,
            connection_checker,
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

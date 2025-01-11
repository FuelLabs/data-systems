use std::{sync::Arc, time::Duration};

use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::stream::*;
use fuel_streams_store::db::{Db, DbConnectionOpts};

use crate::{config::Config, telemetry::Telemetry};

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

#[allow(dead_code)]
#[derive(Clone)]
pub struct Context {
    pub message_broker: Arc<dyn MessageBroker>,
    pub fuel_streams: Arc<FuelStreams>,
    pub telemetry: Arc<Telemetry>,
    pub db: Arc<Db>,
    pub jwt_secret: String,
}

impl Context {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let message_broker =
            MessageBrokerClient::Nats.start(&config.nats.url).await?;
        let db = Db::new(DbConnectionOpts {
            connection_str: config.db.url.clone(),
            ..Default::default()
        })
        .await?
        .arc();

        let fuel_streams = FuelStreams::new(&message_broker, &db).await.arc();
        let telemetry = Telemetry::new(None).await?;
        telemetry.start().await?;

        Ok(Context {
            db,
            fuel_streams,
            message_broker,
            telemetry,
            jwt_secret: config.auth.jwt_secret.clone(),
        })
    }

    #[allow(dead_code)]
    async fn shutdown_services_with_timeout(&self) -> anyhow::Result<()> {
        tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, async {
            Context::flush_await_all_streams(&self.message_broker).await;
        })
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn flush_await_all_streams(message_broker: &Arc<dyn MessageBroker>) {
        tracing::info!("Flushing in-flight messages to nats ...");
        match message_broker.flush().await {
            Ok(_) => {
                tracing::info!("Flushed all streams successfully!");
            }
            Err(e) => {
                tracing::error!("Failed to flush all streams: {:?}", e);
            }
        }
    }
}

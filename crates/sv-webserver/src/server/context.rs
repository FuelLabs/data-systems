use std::{sync::Arc, time::Duration};

use fuel_streams_core::{nats::*, stream::*};
use fuel_streams_store::db::{Db, DbConnectionOpts};

use crate::{config::Config, telemetry::Telemetry};

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

#[allow(dead_code)]
#[derive(Clone)]
pub struct Context {
    pub nats_client: NatsClient,
    pub fuel_streams: Arc<FuelStreams>,
    pub telemetry: Arc<Telemetry>,
    pub db: Arc<Db>,
    pub jwt_secret: String,
}

impl Context {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let nats_client_opts =
            NatsClientOpts::admin_opts().with_url(config.nats.url.clone());
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let db = Db::new(DbConnectionOpts {
            connection_str: config.db.url.clone(),
            ..Default::default()
        })
        .await?
        .arc();

        let fuel_streams = FuelStreams::new(&nats_client, &db).await.arc();
        let telemetry = Telemetry::new(None).await?;
        telemetry.start().await?;

        Ok(Context {
            db,
            fuel_streams,
            nats_client,
            telemetry,
            jwt_secret: config.auth.jwt_secret.clone(),
        })
    }

    #[allow(dead_code)]
    async fn shutdown_services_with_timeout(&self) -> anyhow::Result<()> {
        tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, async {
            Context::flush_await_all_streams(&self.nats_client).await;
        })
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn flush_await_all_streams(nats_client: &NatsClient) {
        tracing::info!("Flushing in-flight messages to nats ...");
        match nats_client.nats_client.flush().await {
            Ok(_) => {
                tracing::info!("Flushed all streams successfully!");
            }
            Err(e) => {
                tracing::error!("Failed to flush all streams: {:?}", e);
            }
        }
    }
}

use std::{sync::Arc, time::Duration};

use fuel_streams_core::prelude::*;
use fuel_streams_storage::S3Client;

use crate::{
    config::Config,
    server::ws::fuel_streams::FuelStreams,
    telemetry::Telemetry,
};

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

#[allow(dead_code)]
#[derive(Clone)]
pub struct Context {
    pub nats_client: NatsClient,
    pub fuel_streams: Arc<FuelStreams>,
    pub telemetry: Arc<Telemetry>,
    pub s3_client: Option<Arc<S3Client>>,
    pub jwt_secret: String,
}

impl Context {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::new(config.nats.network);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let s3_client_opts = S3ClientOpts::admin_opts();
        let s3_client = Arc::new(S3Client::new(&s3_client_opts).await?);
        let fuel_streams =
            Arc::new(FuelStreams::new(&nats_client, &s3_client).await);
        let telemetry = Telemetry::new(None).await?;
        telemetry.start().await?;

        Ok(Context {
            fuel_streams,
            nats_client,
            // client,
            telemetry,
            s3_client: if config.s3.enabled {
                Some(s3_client)
            } else {
                None
            },
            jwt_secret: config.auth.jwt_secret.clone(),
        })
    }

    pub async fn new_for_testing(
        fuel_network: FuelNetwork,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::new(fuel_network);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let s3_client_opts = S3ClientOpts::admin_opts();
        let s3_client = Arc::new(S3Client::new(&s3_client_opts).await?);
        Ok(Context {
            fuel_streams: Arc::new(
                FuelStreams::new(&nats_client, &s3_client).await,
            ),
            nats_client: nats_client.clone(),
            telemetry: Telemetry::new(None).await?,
            s3_client: None,
            jwt_secret: String::new(),
        })
    }

    pub fn get_streams(&self) -> &FuelStreams {
        &self.fuel_streams
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

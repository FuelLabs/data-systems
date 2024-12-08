use std::{sync::Arc, time::Duration};

use aws_sdk_s3::Client as S3Client;
use fuel_streams_core::prelude::*;

use crate::{
    config::Config,
    server::ws::streams::Streams,
    systems::s3::s3_connect,
    telemetry::Telemetry,
};

pub const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(90);

#[allow(dead_code)]
#[derive(Clone)]
pub struct Context {
    pub nats_client: Arc<NatsClient>,
    pub streams: Arc<Streams>,
    pub telemetry: Arc<Telemetry>,
    pub s3_client: Option<Arc<S3Client>>,
    pub jwt_secret: String,
}

impl Context {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::default_opts(None)
            .with_custom_url(config.nats.url.clone());
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let streams = Arc::new(Streams::new(&nats_client).await);
        let telemetry = Telemetry::new(None).await?;
        telemetry.start().await?;

        Ok(Context {
            streams,
            nats_client: Arc::new(nats_client),
            telemetry,
            s3_client: if config.s3.enabled {
                Some(Arc::new(s3_connect(config.s3.clone()).await))
            } else {
                None
            },
            jwt_secret: config.auth.jwt_secret.clone(),
        })
    }

    pub async fn default(nats_client: &NatsClient) -> anyhow::Result<Self> {
        Ok(Context {
            streams: Arc::new(Streams::new(nats_client).await),
            nats_client: Arc::new(nats_client.clone()),
            telemetry: Telemetry::new(None).await?,
            s3_client: None,
            jwt_secret: String::new(),
        })
    }

    pub fn get_streams(&self) -> &Streams {
        &self.streams
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

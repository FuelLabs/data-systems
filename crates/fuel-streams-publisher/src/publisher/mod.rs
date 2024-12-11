pub mod shutdown;

mod blocks_streams;

use std::sync::Arc;

use anyhow::Context;
use blocks_streams::build_blocks_stream;
use fuel_streams_core::prelude::*;
use futures::StreamExt;

use super::{
    shutdown::{ShutdownToken, GRACEFUL_SHUTDOWN_TIMEOUT},
    telemetry::Telemetry,
};

#[derive(Clone)]
pub struct Publisher {
    pub fuel_core: Arc<dyn FuelCoreLike>,
    pub nats_client: NatsClient,
    pub fuel_streams: Arc<dyn FuelStreamsExt>,
    pub telemetry: Arc<Telemetry>,
}

impl Publisher {
    pub async fn new(
        fuel_core: Arc<dyn FuelCoreLike>,
        nats_url: String,
        telemetry: Arc<Telemetry>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts =
            NatsClientOpts::admin_opts(None).with_custom_url(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let fuel_streams = Arc::new(FuelStreams::new(&nats_client).await);

        telemetry.record_streams_count(
            fuel_core.chain_id(),
            fuel_streams.subjects_wildcards().len(),
        );

        Ok(Publisher {
            fuel_core,
            fuel_streams,
            nats_client,
            telemetry,
        })
    }

    pub fn is_healthy(&self) -> bool {
        // TODO: Update this condition to include more health checks
        self.fuel_core.is_started() && self.nats_client.is_connected()
    }

    #[cfg(feature = "test-helpers")]
    pub async fn default(
        nats_client: &NatsClient,
        fuel_core: Arc<dyn FuelCoreLike>,
    ) -> anyhow::Result<Self> {
        Ok(Publisher {
            fuel_core,
            fuel_streams: Arc::new(FuelStreams::new(nats_client).await),
            nats_client: nats_client.clone(),
            telemetry: Telemetry::new().await?,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub fn get_fuel_streams(&self) -> &Arc<(dyn FuelStreamsExt + 'static)> {
        &self.fuel_streams
    }

    async fn shutdown_services_with_timeout(&self) -> anyhow::Result<()> {
        tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, async {
            Publisher::flush_await_all_streams(&self.nats_client).await;
            self.fuel_core.stop().await;
        })
        .await?;

        Ok(())
    }

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

    const MAX_RETAINED_BLOCKS: u32 = 100;
    pub async fn run(
        &self,
        shutdown_token: ShutdownToken,
        historical: bool,
    ) -> anyhow::Result<()> {
        tracing::info!("Awaiting FuelCore Sync...");

        self.fuel_core
            .await_synced_at_least_once(historical)
            .await?;

        tracing::info!("FuelCore has synced successfully!");
        tracing::info!("Publishing started...");

        let mut blocks_stream = build_blocks_stream(
            &self.fuel_streams,
            &self.fuel_core,
            Self::MAX_RETAINED_BLOCKS,
        );

        loop {
            tokio::select! {
                Some(sealed_block) = blocks_stream.next() => {
                    let sealed_block = sealed_block.context("block streams failed to produce sealed block")?;

                        tracing::info!("Processing blocks stream");

                        let fuel_core = &self.fuel_core;
                        let (block, block_producer) =
                            fuel_core.get_block_and_producer(sealed_block.to_owned());

                        // TODO: Avoid awaiting Offchain DB sync for all streams by grouping in their own service
                        fuel_core
                            .await_offchain_db_sync(&block.id())
                            .await
                            .context("Failed to await Offchain DB sync")?;

                        if let Err(err) = self.publish(sealed_block).await {
                            tracing::error!("Failed to publish block data: {}", err);
                            self.telemetry.record_failed_publishing(self.fuel_core.chain_id(), &block_producer);
                        }

                },
                shutdown = shutdown_token.wait_for_shutdown() => {
                    if shutdown {
                        tracing::info!("Shutdown signal received. Stopping services ...");
                        self.shutdown_services_with_timeout().await?;
                        break;
                    }
                },
            };
        }

        tracing::info!("Publishing stopped successfully!");
        Ok(())
    }

    async fn publish(
        &self,
        sealed_block: FuelCoreSealedBlock,
    ) -> anyhow::Result<()> {
        let metadata = Metadata::new(&self.fuel_core, &sealed_block);
        let payload = Arc::new(BlockPayload::new(
            Arc::clone(&self.fuel_core),
            &sealed_block,
            &metadata,
        )?);
        Executor::<Block>::process_all(payload, &self.fuel_streams).await?;
        Ok(())
    }
}

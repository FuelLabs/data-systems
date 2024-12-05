pub mod fuel_core_like;
pub mod fuel_streams;
pub mod payloads;
pub mod shutdown;

mod blocks_streams;

use std::sync::Arc;

use anyhow::Context;
use blocks_streams::build_blocks_stream;
pub use fuel_core_like::{FuelCore, FuelCoreLike};
pub use fuel_streams::{FuelStreams, FuelStreamsExt};
use fuel_streams_core::prelude::*;
use futures::{future::try_join_all, stream::FuturesUnordered, StreamExt};
use tokio::sync::Semaphore;

use super::{
    payloads::blocks,
    shutdown::{ShutdownToken, GRACEFUL_SHUTDOWN_TIMEOUT},
    telemetry::Telemetry,
    PUBLISHER_MAX_THREADS,
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

    const MAX_RETAINED_BLOCKS: u64 = 100;
    pub async fn run(
        &self,
        shutdown_token: ShutdownToken,
    ) -> anyhow::Result<()> {
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
                            fuel_core.get_block_and_producer(sealed_block);

                        // TODO: Avoid awaiting Offchain DB sync for all streams by grouping in their own service
                        fuel_core
                            .await_offchain_db_sync(&block.id())
                            .await
                            .context("Failed to await Offchain DB sync")?;

                        if let Err(err) = self.publish(&block, &block_producer).await {
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
        block: &FuelCoreBlock<FuelCoreTransaction>,
        block_producer: &Address,
    ) -> anyhow::Result<()> {
        let start_time = std::time::Instant::now();

        let semaphore = Arc::new(Semaphore::new(*PUBLISHER_MAX_THREADS));
        let chain_id = Arc::new(*self.fuel_core.chain_id());
        let base_asset_id = Arc::new(*self.fuel_core.base_asset_id());
        let block_producer = Arc::new(block_producer.clone());
        let block_height = block.header().consensus().height;
        let txs = block.transactions();
        let transaction_ids = txs
            .iter()
            .map(|tx| tx.id(&chain_id).into())
            .collect::<Vec<Bytes32>>();

        let consensus: Consensus =
            self.fuel_core.get_consensus(&block_height)?.into();

        let fuel_core = &*self.fuel_core;
        let offchain_database = fuel_core.offchain_database()?;

        let fuel_streams = &*self.fuel_streams;
        let blocks_stream = Arc::new(fuel_streams.blocks().to_owned());
        let opts = &Arc::new(PublishOpts {
            semaphore,
            chain_id,
            base_asset_id,
            block_producer: Arc::clone(&block_producer),
            block_height: Arc::new(block_height.into()),
            telemetry: self.telemetry.clone(),
            consensus: Arc::new(consensus),
            offchain_database,
        });

        let publish_tasks = payloads::transactions::publish_all_tasks(
            txs,
            fuel_streams,
            opts,
            fuel_core,
        )?
        .into_iter()
        .chain(std::iter::once(blocks::publish_task(
            block,
            blocks_stream,
            opts,
            transaction_ids,
        )))
        .collect::<FuturesUnordered<_>>();

        try_join_all(publish_tasks).await?;

        let elapsed = start_time.elapsed();
        tracing::info!(
            "Published streams for BlockHeight: {} in {:?}",
            *block_height,
            elapsed
        );

        Ok(())
    }
}

use tokio::task::JoinHandle;

use crate::fuel_core_like::OffchainDatabase;

#[derive(Clone)]
pub struct PublishOpts {
    pub semaphore: Arc<Semaphore>,
    pub chain_id: Arc<FuelCoreChainId>,
    pub base_asset_id: Arc<FuelCoreAssetId>,
    pub block_producer: Arc<Address>,
    pub block_height: Arc<BlockHeight>,
    pub telemetry: Arc<Telemetry>,
    pub consensus: Arc<Consensus>,
    pub offchain_database: Arc<OffchainDatabase>,
}

pub fn publish<S: Streamable + 'static>(
    packet: &PublishPacket<S>,
    stream: Arc<Stream<S>>,
    opts: &Arc<PublishOpts>,
) -> JoinHandle<anyhow::Result<()>> {
    let opts = Arc::clone(opts);
    let payload = Arc::clone(&packet.payload);
    let subject = Arc::clone(&packet.subject);
    let telemetry = Arc::clone(&opts.telemetry);
    let wildcard = packet.subject.wildcard();

    tokio::spawn(async move {
        let _permit = opts.semaphore.acquire().await?;

        match stream.publish(&*subject, &payload).await {
            Ok(published_data_size) => {
                telemetry.log_info(&format!(
                    "Successfully published for stream: {}",
                    wildcard
                ));
                telemetry.update_publisher_success_metrics(
                    wildcard,
                    published_data_size,
                    &opts.chain_id,
                    &opts.block_producer,
                );

                Ok(())
            }
            Err(e) => {
                telemetry.log_error(&e.to_string());
                telemetry.update_publisher_error_metrics(
                    wildcard,
                    &opts.chain_id,
                    &opts.block_producer,
                    &e.to_string(),
                );

                anyhow::bail!("Failed to publish: {}", e.to_string())
            }
        }
    })
}

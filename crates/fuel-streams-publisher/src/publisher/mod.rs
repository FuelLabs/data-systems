pub mod fuel_core_like;
pub mod payloads;
pub mod shutdown;
pub mod streams;

use std::sync::Arc;

use fuel_core_importer::ImporterResult;
pub use fuel_core_like::{FuelCore, FuelCoreLike};
use fuel_streams_core::prelude::*;
use futures::{future::try_join_all, stream::FuturesUnordered};
pub use streams::Streams;
use tokio::sync::{broadcast::error::RecvError, Semaphore};

use super::{
    payloads::blocks,
    shutdown::{ShutdownToken, GRACEFUL_SHUTDOWN_TIMEOUT},
    telemetry::Telemetry,
    PUBLISHER_MAX_THREADS,
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Publisher {
    pub fuel_core: Arc<dyn FuelCoreLike>,
    pub nats_client: NatsClient,
    pub streams: Arc<Streams>,
    pub telemetry: Arc<Telemetry>,
}

impl Publisher {
    pub async fn new(
        fuel_core: Arc<dyn FuelCoreLike>,
        nats_url: &str,
        telemetry: Arc<Telemetry>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let streams = Arc::new(Streams::new(&nats_client).await);

        telemetry.record_streams_count(
            fuel_core.chain_id(),
            streams.subjects_wildcards().len(),
        );

        Ok(Publisher {
            fuel_core,
            streams,
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
            streams: Arc::new(Streams::new(nats_client).await),
            nats_client: nats_client.clone(),
            telemetry: Telemetry::new().await?,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub fn get_streams(&self) -> &Streams {
        &self.streams
    }

    async fn publish_block_data(
        &self,
        result: ImporterResult,
    ) -> anyhow::Result<()> {
        let (block, block_producer) =
            self.fuel_core.get_block_and_producer(&result.sealed_block);
        self.publish(&block, &block_producer).await?;
        Ok(())
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

    pub async fn run(
        &self,
        shutdown_token: ShutdownToken,
    ) -> anyhow::Result<()> {
        let last_published_block = self
            .streams
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await?;
        let last_published_height: u64 = last_published_block
            .map(|block| block.height.into())
            .unwrap_or(0);

        // Catch up the streams with the FuelCore
        if let Some(latest_fuel_core_height) =
            self.fuel_core.get_latest_block_height()?
        {
            if latest_fuel_core_height > last_published_height + 1 {
                tracing::warn!("Missing blocks: last block height in Node={latest_fuel_core_height}, last published block height={last_published_height}");
            }

            let latest_fuel_block_time_unix = self
                .fuel_core
                .get_sealed_block_time_by_height(latest_fuel_core_height as u32)
                .to_unix();

            let mut height = last_published_height;
            while height <= latest_fuel_core_height {
                tokio::select! {
                    shutdown =  shutdown_token.wait_for_shutdown() => {
                        if shutdown {
                            tracing::info!("Shutdown signal received during historical blocks processing. Last published block height {height}");
                            self.shutdown_services_with_timeout().await?;
                            return Ok(());
                        }
                    },
                    (result, block_producer) = async {

                        let fuel_block_time_unix = self
                        .fuel_core
                        .get_sealed_block_time_by_height(height as u32)
                        .to_unix();

                        if fuel_block_time_unix < latest_fuel_block_time_unix - (FUEL_BLOCK_TIME_SECS  * MAX_RETENTION_BLOCKS) as i64 {
                            // Skip publishing for this block and move to the next height
                            tracing::warn!("Block {} with time: {} is more than 100 blocks behind chain tip, skipped publishing", height, fuel_block_time_unix);
                            return (Ok(()), None);
                        }

                        let sealed_block = self
                            .fuel_core
                            .get_sealed_block_by_height(height as u32);

                        let (block, block_producer) =
                            self.fuel_core.get_block_and_producer(&sealed_block);

                        (self.publish(&block, &block_producer).await, Some(block_producer.clone()))
                    } => {
                        if let (Err(err), Some(block_producer)) = (result, block_producer) {
                            tracing::warn!("Failed to publish block data: {}", err);
                            self.telemetry.record_failed_publishing(self.fuel_core.chain_id(), &block_producer);
                        }
                        // Increment the height after processing
                        height += 1;
                    }
                }
            }
        }

        // publish subscribed data
        loop {
            let fuel_core = self.fuel_core.clone();
            let mut blocks_subscription = fuel_core.blocks_subscription();

            tokio::select! {
                result = blocks_subscription.recv() => {
                    match result {
                        Ok(result) => {
                            self.publish_block_data(result).await?;
                        }
                        Err(RecvError::Closed) | Err(RecvError::Lagged(_)) => {
                            // The sender has been dropped or has lagged, exit the loop
                            break;
                        }
                    }
                }
                shutdown = shutdown_token.wait_for_shutdown()  => {
                    if shutdown {
                        self.shutdown_services_with_timeout().await?;
                        break;
                    }
                }
            }
        }

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
        let consensus: Consensus =
            self.fuel_core.get_consensus(&block_height)?.into();

        let fuel_core = &*self.fuel_core;
        let offchain_database = fuel_core.offchain_database()?;

        let streams = (*self.streams).clone();
        let block_stream = Arc::new(streams.blocks.to_owned());
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
            txs, streams, opts, fuel_core,
        )
        .into_iter()
        .chain(std::iter::once(blocks::publish_task(
            block,
            block_stream,
            opts,
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
    pub chain_id: Arc<ChainId>,
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
    let stream = Arc::clone(&stream);
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

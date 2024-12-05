pub mod fuel_core_like;
pub mod payloads;
pub mod shutdown;
pub mod streams;

use std::{cmp::max, pin::Pin, sync::Arc};

use anyhow::Context;
pub use fuel_core_like::{FuelCore, FuelCoreLike};
use fuel_core_types::blockchain::SealedBlock;
use fuel_streams_core::prelude::*;
use futures::{
    future::try_join_all,
    stream::{self, BoxStream, FuturesUnordered},
    StreamExt,
    TryStreamExt,
};
pub use streams::Streams;
use tokio::sync::Semaphore;
use tokio_stream::wrappers::BroadcastStream;

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
    pub streams: Arc<Streams>,
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
        tracing::info!("Publishing started...");

        let mut stream_of_blocks_stream = self.get_blocks_stream();

        loop {
            tokio::select! {
                Some(sealed_block) = stream_of_blocks_stream.next() => {

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

    fn get_blocks_stream<'a>(
        &'a self,
    ) -> BoxStream<'a, anyhow::Result<SealedBlock>> {
        let this = Arc::new(self.clone());

        stream::try_unfold((), move |_state| {
            let this = Arc::clone(&this);
            let fuel_core = this.fuel_core.clone();

            async move {
                let latest_block_height =
                    fuel_core.get_latest_block_height()?;
                let last_published_block_height = this
                    .get_last_published_block_height(latest_block_height)
                    .await?;

                let old_blocks_stream = stream::iter(
                    last_published_block_height..latest_block_height,
                )
                .map({
                    let fuel_core = fuel_core.clone();

                    move |height| {
                        (&fuel_core).get_sealed_block_by_height(height as u32)
                    }
                });

                let has_published_latest =
                    latest_block_height == last_published_block_height;

                let blocks_stream = if has_published_latest {
                    BroadcastStream::new(fuel_core.blocks_subscription())
                        .map(|import_result| {
                            import_result
                                .expect("Failed to get ImportResult")
                                .sealed_block
                                .clone()
                        })
                        .map(Ok)
                        .boxed()
                } else {
                    old_blocks_stream.map(Ok).boxed()
                };

                anyhow::Ok(Some((blocks_stream, ())))
            }
        })
        .try_flatten()
        .boxed()
    }

    const MAX_RETAINED_BLOCKS: i64 = 100;
    async fn get_last_published_block_height(
        &self,
        latest_block_height: u64,
    ) -> anyhow::Result<u64> {
        let max_last_published_block_height =
            max(0, latest_block_height as i64 - Self::MAX_RETAINED_BLOCKS)
                as u64;

        Ok(self
            .streams
            .get_last_published_block()
            .await?
            .map(|block| block.height.into())
            .map(|block_height: u64| {
                max(block_height, max_last_published_block_height)
            })
            .unwrap_or(max_last_published_block_height))
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
        )?
        .into_iter()
        .chain(std::iter::once(blocks::publish_task(
            block,
            block_stream,
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

use std::{sync::Arc, time::Duration};

use fuel_core::{
    combined_database::CombinedDatabase,
    database::database_description::DatabaseHeight,
};
use fuel_core_bin::FuelService;
use fuel_core_importer::ImporterResult;
use fuel_core_services::Service;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::blockchain::consensus::Sealed;
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{NatsClient, NatsClientOpts},
    prelude::IntoSubject,
    types::{Address, AssetId, Block, BlockHeight, ChainId, Transaction},
    Stream,
};
use futures_util::{future::try_join_all, FutureExt};
use tokio::sync::broadcast::Receiver;
use tracing::warn;

use crate::{blocks, shutdown::stop_signal, state::SharedState, transactions};

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Streams we currently support publishing to.
pub struct Streams {
    pub transactions: Stream<Transaction>,
    pub blocks: Stream<Block>,
}

impl Streams {
    pub async fn new(nats_client: &NatsClient) -> Self {
        Self {
            transactions: Stream::<Transaction>::new(nats_client).await,
            blocks: Stream::<Block>::new(nats_client).await,
        }
    }

    #[cfg(feature = "test-helpers")]
    pub async fn is_empty(&self) -> bool {
        use fuel_streams_core::transactions::TransactionsSubject;

        self.blocks.is_empty(BlocksSubject::WILDCARD).await
            && self
                .transactions
                .is_empty(TransactionsSubject::WILDCARD)
                .await
    }
}

#[allow(dead_code)]
/// TODO: Remove right after using chain_id and base_asset_id to publish
/// TransactionsById subject
pub struct Publisher {
    fuel_service: Arc<FuelService>,
    chain_id: ChainId,
    base_asset_id: AssetId,
    fuel_core_database: CombinedDatabase,
    blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    streams: Streams,
}

impl Publisher {
    pub async fn new(
        fuel_service: Arc<FuelService>,
        nats_url: &str,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;

        let fuel_core_subscription = fuel_service
            .shared
            .block_importer
            .block_importer
            .subscribe();
        let fuel_core_database = fuel_service.shared.database.clone();

        let chain_config =
            fuel_service.shared.config.snapshot_reader.chain_config();
        let chain_id = chain_config.consensus_parameters.chain_id();
        let base_asset_id =
            chain_config.consensus_parameters.base_asset_id().clone();

        Ok(Publisher {
            fuel_service,
            chain_id,
            base_asset_id,
            fuel_core_database,
            blocks_subscription: fuel_core_subscription,
            streams: Streams::new(&nats_client).await,
        })
    }

    pub async fn flush_await_all_streams(&self) -> anyhow::Result<()> {
        let streams = [
            self.streams.blocks.flush_await().boxed(),
            self.streams.transactions.flush_await().boxed(),
        ];
        try_join_all(streams).await?;
        Ok(())
    }

    #[cfg(feature = "test-helpers")]
    pub async fn default_with_publisher(
        fuel_service: Arc<FuelService>,
        nats_client: &NatsClient,
        blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    ) -> anyhow::Result<Self> {
        Ok(Publisher {
            fuel_service,
            chain_id: ChainId::default(),
            base_asset_id: AssetId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            streams: Streams::new(nats_client).await,
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
            Self::get_block_and_producer(&result.sealed_block);
        self.publish(&block, &block_producer).await?;
        Ok(())
    }

    async fn shutdown_services_with_timeout(&self) -> anyhow::Result<()> {
        tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, async {
            tracing::info!("Flushing in-flight messages to nats ...");
            match self.flush_await_all_streams().await {
                Ok(_) => tracing::info!("Flushed in-flight messages to nats"),
                Err(e) => tracing::error!(
                    "Flushing in-flight messages to nats failed: {:?}",
                    e
                ),
            }

            tracing::info!("Stopping fuel core ...");
            match self.fuel_service.stop_and_await().await {
                Ok(_) => tracing::info!("Stopped fuel core"),
                Err(e) => tracing::error!("Stopping fuel core failed: {:?}", e),
            }
        })
        .await?;

        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<Self> {
        let (shutdown_send, mut shutdown_recv) =
            tokio::sync::broadcast::channel::<()>(1);
        tokio::spawn(stop_signal(shutdown_send));

        let last_published_block = self
            .streams
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await?;
        let last_published_height = last_published_block
            .map(|block| block.header().height().as_u64())
            .unwrap_or(0);
        let next_height_to_publish = last_published_height + 1;

        // Catch up the streams with the FuelCore
        if let Some(latest_fuel_core_height) = self
            .fuel_core_database
            .on_chain()
            .latest_height()?
            .map(|h| h.as_u64())
        {
            if latest_fuel_core_height > last_published_height + 1 {
                warn!("Missing blocks: last block height in Node={latest_fuel_core_height}, last published block height={last_published_height}");
            }

            // publish historical data i needed
            let mut height = next_height_to_publish;
            while height <= latest_fuel_core_height {
                tokio::select! {
                    _ = shutdown_recv.recv() => {
                        tracing::info!("Shutdown signal received during historical blocks processing. Last published block height {height}");
                        self.shutdown_services_with_timeout().await?;
                        return Ok(self);
                    },
                    result = async {
                        let sealed_block = self
                            .fuel_core_database
                            .on_chain()
                            .latest_view()?
                            .get_sealed_block_by_height(&(height as u32).into())?
                            .expect("NATS Publisher: no block at height {height}");

                        let (block, block_producer) =
                            Self::get_block_and_producer(&sealed_block);

                        self.publish(&block, &block_producer).await
                    } => {
                        if let Err(err) = result {
                            tracing::warn!("Failed to publish block data: {}", err);
                        }
                        height += 1;
                    }
                }
            }
        }

        // publish subscribed data
        loop {
            tokio::select! {
                result = self.blocks_subscription.recv() => {
                    if let Ok(result) = result {
                        self.publish_block_data(result).await?;
                    }
                }
                _ = shutdown_recv.recv() => {
                    self.shutdown_services_with_timeout().await?;
                    break;
                }
            }
        }

        Ok(self)
    }

    #[cfg(not(feature = "test-helpers"))]
    fn get_block_and_producer(
        sealed_block: &Sealed<Block>,
    ) -> (Block, Address) {
        let block = sealed_block.entity.clone();
        let block_producer = sealed_block
            .consensus
            .block_producer(&block.id())
            .expect("Failed to get Block Producer");

        (block, block_producer.into())
    }

    async fn publish(
        &self,
        block: &Block<Transaction>,
        block_producer: &Address,
    ) -> anyhow::Result<()> {
        let block_height: BlockHeight =
            block.header().consensus().height.into();

        blocks::publish(
            &block_height,
            &self.streams.blocks,
            block,
            block_producer,
        )
        .await?;

        transactions::publish(
            &self.chain_id,
            &block_height,
            &self.fuel_core_database,
            &self.streams.transactions,
            block.transactions(),
        )
        .await?;

        Ok(())
    }

    #[cfg(feature = "test-helpers")]
    fn get_block_and_producer(
        sealed_block: &Sealed<Block>,
    ) -> (Block, Address) {
        let block = sealed_block.entity.clone();
        let block_producer = sealed_block
            .consensus
            .block_producer(&block.id())
            .unwrap_or_default();

        (block, block_producer.into())
    }
}

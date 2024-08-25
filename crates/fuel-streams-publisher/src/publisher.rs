use fuel_core::{
    combined_database::CombinedDatabase,
    database::database_description::DatabaseHeight,
};
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::blockchain::consensus::Sealed;
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{NatsClient, NatsClientOpts},
    prelude::IntoSubject,
    types::{Address, AssetId, Block, BlockHeight, ChainId, Transaction},
    Stream,
};
use tokio::sync::broadcast::Receiver;
use tracing::warn;

use crate::{blocks, transactions};

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
    chain_id: ChainId,
    base_asset_id: AssetId,
    fuel_core_database: CombinedDatabase,
    blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    streams: Streams,
}

impl Publisher {
    pub async fn new(
        nats_url: &str,
        chain_id: ChainId,
        base_asset_id: AssetId,
        fuel_core_database: CombinedDatabase,
        blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;

        Ok(Publisher {
            chain_id,
            base_asset_id,
            fuel_core_database,
            blocks_subscription,
            streams: Streams::new(&nats_client).await,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub async fn default_with_publisher(
        nats_client: &NatsClient,
        blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    ) -> anyhow::Result<Self> {
        Ok(Publisher {
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

    pub async fn run(mut self) -> anyhow::Result<Self> {
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

            for height in next_height_to_publish..=latest_fuel_core_height {
                let sealed_block = self
                    .fuel_core_database
                    .on_chain()
                    .latest_view()?
                    .get_sealed_block_by_height(&(height as u32).into())?
                    .expect("NATS Publisher: no block at height {height}");

                let (block, block_producer) =
                    Self::get_block_and_producer(&sealed_block);

                self.publish(&block, &block_producer).await?;
            }
        }

        while let Ok(result) = self.blocks_subscription.recv().await {
            let (block, block_producer) =
                Self::get_block_and_producer(&result.sealed_block);

            self.publish(&block, &block_producer).await?;
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

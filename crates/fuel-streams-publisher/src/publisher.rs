<<<<<<< HEAD
use fuel_core::database::database_description::DatabaseHeight;
use fuel_core_types::fuel_tx::Output;
||||||| parent of b1d06dd (feat(publisher): publish outputs)
use fuel_core::{
    combined_database::CombinedDatabase,
    database::database_description::DatabaseHeight,
};
use fuel_core_importer::ports::ImporterDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::blockchain::consensus::Sealed;
=======
use fuel_core::{
    combined_database::CombinedDatabase,
    database::database_description::DatabaseHeight,
};
use fuel_core_importer::ports::ImporterDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::{blockchain::consensus::Sealed, fuel_tx::output::Output};
>>>>>>> b1d06dd (feat(publisher): publish outputs)
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::NatsClient,
    types::{Address, Block, BlockHeight, Input, Receipt, Transaction},
    Stream,
};
use tracing::warn;

use crate::{blocks, inputs, outputs, receipts, transactions, FuelCoreLike};

/// Streams we currently support publishing to.
pub struct Streams {
    pub transactions: Stream<Transaction>,
    pub blocks: Stream<Block>,
<<<<<<< HEAD
    pub inputs: Stream<Input>,
    pub outputs: Stream<Output>,
    pub receipts: Stream<Receipt>,
||||||| parent of b1d06dd (feat(publisher): publish outputs)
=======
    pub outputs: Stream<Output>,
>>>>>>> b1d06dd (feat(publisher): publish outputs)
}

impl Streams {
    pub async fn new(nats_client: &NatsClient) -> Self {
        Self {
            transactions: Stream::<Transaction>::new(nats_client).await,
            blocks: Stream::<Block>::new(nats_client).await,
<<<<<<< HEAD
            inputs: Stream::<Input>::new(nats_client).await,
            outputs: Stream::<Output>::new(nats_client).await,
            receipts: Stream::<Receipt>::new(nats_client).await,
||||||| parent of b1d06dd (feat(publisher): publish outputs)
=======
            outputs: Stream::<Output>::new(nats_client).await,
>>>>>>> b1d06dd (feat(publisher): publish outputs)
        }
    }

    #[cfg(feature = "test-helpers")]
    pub async fn is_empty(&self) -> bool {
        use fuel_streams_core::{
            outputs::OutputsSubject,
            transactions::TransactionsSubject,
        };

        self.blocks.is_empty(BlocksSubject::WILDCARD).await
            && self
                .transactions
                .is_empty(TransactionsSubject::WILDCARD)
                .await
            && self.outputs.is_empty(OutputsSubject::WILDCARD).await
    }
}

#[allow(dead_code)]
pub struct Publisher {
    streams: Streams,
    fuel_core: Box<dyn FuelCoreLike>,
}

impl Publisher {
    pub async fn new(
        nats_client: &NatsClient,
        fuel_core: Box<dyn FuelCoreLike>,
    ) -> Self {
        Self {
            fuel_core,
            streams: Streams::new(nats_client).await,
        }
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
        if let Some(latest_fuel_core_height) =
            self.fuel_core.get_latest_block_height()?
        {
            if latest_fuel_core_height > last_published_height + 1 {
                warn!("Missing blocks: last block height in Node={latest_fuel_core_height}, last published block height={last_published_height}");
            }

            for height in next_height_to_publish..=latest_fuel_core_height {
                let (block, block_producer) =
                    self.fuel_core.get_block_and_producer_by_height(height)?;

                self.publish(&block, &block_producer).await?;
            }
        }

        while let Ok(result) = self.fuel_core.blocks_subscription().recv().await
        {
            let (block, block_producer) =
                self.fuel_core.get_block_and_producer(&result.sealed_block);

            self.publish(&block, &block_producer).await?;
        }

        Ok(self)
    }

    async fn publish(
        &self,
        block: &Block<Transaction>,
        block_producer: &Address,
    ) -> anyhow::Result<()> {
        let block_height: BlockHeight =
            block.header().consensus().height.into();
        let transactions = block.transactions();

        blocks::publish(
            &block_height,
            &self.streams.blocks,
            block,
            block_producer,
        )
        .await?;

        transactions::publish(
            &block_height,
            &*self.fuel_core,
            &self.streams.transactions,
            transactions,
        )
        .await?;

        receipts::publish(
            &*self.fuel_core,
            &self.streams.receipts,
            transactions,
        )
        .await?;

        inputs::publish(
            &self.streams.inputs,
            self.fuel_core.chain_id(),
            transactions,
        )
        .await?;

        outputs::publish(
            &self.streams.outputs,
            self.fuel_core.chain_id(),
            transactions,
        )
        .await?;

        outputs::publish(
            &self.streams.outputs,
            &self.chain_id,
            block.transactions(),
        )
        .await?;

        Ok(())
    }
}

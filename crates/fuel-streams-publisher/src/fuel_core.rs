use fuel_core::{
    combined_database::CombinedDatabase,
    database::database_description::DatabaseHeight,
};
use fuel_core_importer::ports::ImporterDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::{
    blockchain::consensus::Sealed,
    fuel_tx::Bytes32,
    tai64::Tai64,
};
use fuel_streams_core::types::{
    Address,
    Block,
    ChainId,
    FuelCoreTransactionStatus,
    Receipt,
};
use tokio::sync::broadcast::Receiver;

/// Interface for `fuel-core` related logic.
/// This was introduced to simplify mocking and testing the `fuel-streams-publisher` crate.
#[async_trait::async_trait]
pub trait FuelCoreLike: Sync + Send {
    fn chain_id(&self) -> &ChainId;
    fn database(&self) -> &CombinedDatabase;
    fn blocks_subscription(
        &mut self,
    ) -> &mut Receiver<fuel_core_importer::ImporterResult>;

    fn get_latest_block_height(&self) -> anyhow::Result<Option<u64>> {
        Ok(self
            .database()
            .on_chain()
            .latest_block_height()?
            .map(|h| h.as_u64()))
    }

    fn get_receipts(
        &self,
        tx_id: &Bytes32,
    ) -> anyhow::Result<Option<Vec<Receipt>>>;

    fn get_block_and_producer_by_height(
        &self,
        height: u64,
    ) -> anyhow::Result<(Block, Address)> {
        let sealed_block = self
            .database()
            .on_chain()
            .latest_view()?
            .get_sealed_block_by_height(&(height as u32).into())?
            .expect("NATS Publisher: no block at height {height}");

        Ok(self.get_block_and_producer(&sealed_block))
    }

    #[cfg(not(feature = "test-helpers"))]
    fn get_block_and_producer(
        &self,
        sealed_block: &Sealed<Block>,
    ) -> (Block, Address) {
        let block = sealed_block.entity.clone();
        let block_producer = sealed_block
            .consensus
            .block_producer(&block.id())
            .expect("Failed to get Block Producer");

        (block, block_producer.into())
    }

    #[cfg(feature = "test-helpers")]
    fn get_block_and_producer(
        &self,
        sealed_block: &Sealed<Block>,
    ) -> (Block, Address) {
        let block = sealed_block.entity.clone();
        let block_producer = sealed_block
            .consensus
            .block_producer(&block.id())
            .unwrap_or_default();

        (block, block_producer.into())
    }

    fn get_sealed_block_by_height(&self, height: u32) -> Sealed<Block> {
        self.database()
            .on_chain()
            .latest_view()
            .expect("failed to get latest db view")
            .get_sealed_block_by_height(&height.into())
            .expect("Failed to get latest block height")
            .expect("NATS Publisher: no block at height {height}")
    }

    fn get_sealed_block_time_by_height(&self, height: u32) -> Tai64 {
        self.database()
            .on_chain()
            .latest_view()
            .expect("failed to get latest db view")
            .get_sealed_block_header(&height.into())
            .expect("Failed to get sealed block header")
            .expect("Failed to find sealed block header")
            .entity
            .time()
    }
}

pub struct FuelCore {
    database: CombinedDatabase,
    blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    chain_id: ChainId,
}

impl FuelCore {
    pub async fn new(
        database: CombinedDatabase,
        blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
        chain_id: ChainId,
    ) -> Box<Self> {
        Box::new(Self {
            database,
            blocks_subscription,
            chain_id,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub async fn from_blocks_subscription(
        blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    ) -> Box<Self> {
        Box::new(Self {
            chain_id: ChainId::default(),
            database: CombinedDatabase::default(),
            blocks_subscription,
        })
    }
}

#[async_trait::async_trait]
impl FuelCoreLike for FuelCore {
    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    fn database(&self) -> &CombinedDatabase {
        &self.database
    }

    fn blocks_subscription(
        &mut self,
    ) -> &mut Receiver<fuel_core_importer::ImporterResult> {
        &mut self.blocks_subscription
    }

    fn get_receipts(
        &self,
        tx_id: &Bytes32,
    ) -> anyhow::Result<Option<Vec<Receipt>>> {
        let off_chain_database = self.database().off_chain().latest_view()?;
        let receipts = off_chain_database
            .get_tx_status(tx_id)?
            .map(|status| match &status {
                FuelCoreTransactionStatus::Success { receipts, .. } => {
                    Some(receipts.clone())
                }
                FuelCoreTransactionStatus::Failed { receipts, .. } => {
                    Some(receipts.clone())
                }
                _ => None,
            })
            .unwrap_or_default();

        Ok(receipts)
    }
}

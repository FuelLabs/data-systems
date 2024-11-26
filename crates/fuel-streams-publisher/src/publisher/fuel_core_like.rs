use std::sync::Arc;

use fuel_core::{
    combined_database::CombinedDatabase,
    database::{
        database_description::{on_chain::OnChain, DatabaseHeight},
        Database,
    },
    fuel_core_graphql_api::ports::DatabaseBlocks,
    state::{
        generic_database::GenericDatabase,
        iterable_key_value_view::IterableKeyValueViewWrapper,
    },
};
use fuel_core_bin::FuelService;
use fuel_core_importer::ports::ImporterDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::{
    blockchain::consensus::{Consensus, Sealed},
    fuel_types::BlockHeight,
    tai64::Tai64,
};
use fuel_streams_core::types::*;
use tokio::sync::broadcast::Receiver;

pub type OffchainDatabase = GenericDatabase<
    IterableKeyValueViewWrapper<
        fuel_core::fuel_core_graphql_api::storage::Column,
    >,
>;

/// Interface for `fuel-core` related logic.
/// This was introduced to simplify mocking and testing the `fuel-streams-publisher` crate.
#[async_trait::async_trait]
pub trait FuelCoreLike: Sync + Send {
    async fn start(&self);
    fn is_started(&self) -> bool;
    async fn stop(&self);

    fn base_asset_id(&self) -> &FuelCoreAssetId;
    fn chain_id(&self) -> &FuelCoreChainId;

    fn database(&self) -> &CombinedDatabase;
    fn onchain_database(&self) -> &Database<OnChain> {
        self.database().on_chain()
    }
    fn offchain_database(&self) -> anyhow::Result<Arc<OffchainDatabase>> {
        Ok(Arc::new(self.database().off_chain().latest_view()?))
    }

    fn blocks_subscription(
        &self,
    ) -> Receiver<fuel_core_importer::ImporterResult>;

    fn get_latest_block_height(&self) -> anyhow::Result<Option<u64>> {
        Ok(self
            .onchain_database()
            .latest_block_height()?
            .map(|h| h.as_u64()))
    }

    fn get_receipts(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> anyhow::Result<Option<Vec<FuelCoreReceipt>>>;

    fn get_block_and_producer_by_height(
        &self,
        height: u32,
    ) -> anyhow::Result<(FuelCoreBlock, Address)> {
        let sealed_block = self
            .onchain_database()
            .latest_view()?
            .get_sealed_block_by_height(&(height).into())?
            .expect("NATS Publisher: no block at height {height}");

        Ok(self.get_block_and_producer(&sealed_block))
    }

    #[cfg(not(feature = "test-helpers"))]
    fn get_consensus(
        &self,
        block_height: &BlockHeight,
    ) -> anyhow::Result<Consensus> {
        Ok(self
            .onchain_database()
            .latest_view()?
            .consensus(block_height)?)
    }

    #[cfg(feature = "test-helpers")]
    fn get_consensus(
        &self,
        block_height: &BlockHeight,
    ) -> anyhow::Result<Consensus> {
        Ok(self
            .onchain_database()
            .latest_view()?
            .consensus(block_height)
            .unwrap_or_default())
    }

    #[cfg(not(feature = "test-helpers"))]
    fn get_block_and_producer(
        &self,
        sealed_block: &Sealed<FuelCoreBlock>,
    ) -> (FuelCoreBlock, Address) {
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
        sealed_block: &Sealed<FuelCoreBlock>,
    ) -> (FuelCoreBlock, Address) {
        let block = sealed_block.entity.clone();
        let block_producer = sealed_block
            .consensus
            .block_producer(&block.id())
            .unwrap_or_default();

        (block, block_producer.into())
    }

    fn get_sealed_block_by_height(&self, height: u32) -> Sealed<FuelCoreBlock> {
        self.onchain_database()
            .latest_view()
            .expect("failed to get latest db view")
            .get_sealed_block_by_height(&height.into())
            .expect("Failed to get latest block height")
            .expect("NATS Publisher: no block at height {height}")
    }

    fn get_sealed_block_time_by_height(&self, height: u32) -> Tai64 {
        self.onchain_database()
            .latest_view()
            .expect("failed to get latest db view")
            .get_sealed_block_header(&height.into())
            .expect("Failed to get sealed block header")
            .expect("Failed to find sealed block header")
            .entity
            .time()
    }
}

#[derive(Clone)]
pub struct FuelCore {
    pub fuel_service: Arc<FuelService>,
    chain_id: FuelCoreChainId,
    base_asset_id: FuelCoreAssetId,
    database: CombinedDatabase,
}

impl From<FuelService> for FuelCore {
    fn from(fuel_service: FuelService) -> Self {
        let chain_config =
            fuel_service.shared.config.snapshot_reader.chain_config();
        let chain_id = chain_config.consensus_parameters.chain_id();
        let base_asset_id = *chain_config.consensus_parameters.base_asset_id();

        let database = fuel_service.shared.database.clone();

        Self {
            fuel_service: Arc::new(fuel_service),
            chain_id,
            base_asset_id,
            database,
        }
    }
}

impl FuelCore {
    pub async fn new(
        command: fuel_core_bin::cli::run::Command,
    ) -> anyhow::Result<Arc<Self>> {
        let fuel_service =
            fuel_core_bin::cli::run::get_service(command).await?;

        let fuel_core: Self = fuel_service.into();

        Ok(fuel_core.arc())
    }
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait::async_trait]
impl FuelCoreLike for FuelCore {
    async fn start(&self) {
        fuel_core_bin::cli::init_logging();

        self.fuel_service
            .start_and_await()
            .await
            .expect("Fuel core service startup failed");
    }

    fn is_started(&self) -> bool {
        self.fuel_service.state().started()
    }

    async fn stop(&self) {
        if matches!(
            self.fuel_service.state(),
            fuel_core_services::State::Stopped
                | fuel_core_services::State::Stopping
                | fuel_core_services::State::StoppedWithError(_)
                | fuel_core_services::State::NotStarted
        ) {
            return;
        }

        tracing::info!("Stopping fuel core ...");
        match self
            .fuel_service
            .send_stop_signal_and_await_shutdown()
            .await
        {
            Ok(state) => {
                tracing::info!("Stopped fuel core. Status = {:?}", state)
            }
            Err(e) => tracing::error!("Stopping fuel core failed: {:?}", e),
        }
    }

    fn base_asset_id(&self) -> &FuelCoreAssetId {
        &self.base_asset_id
    }
    fn chain_id(&self) -> &FuelCoreChainId {
        &self.chain_id
    }

    fn database(&self) -> &CombinedDatabase {
        &self.database
    }

    fn blocks_subscription(
        &self,
    ) -> Receiver<fuel_core_importer::ImporterResult> {
        self.fuel_service
            .shared
            .block_importer
            .block_importer
            .subscribe()
    }

    fn get_receipts(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> anyhow::Result<Option<Vec<FuelCoreReceipt>>> {
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

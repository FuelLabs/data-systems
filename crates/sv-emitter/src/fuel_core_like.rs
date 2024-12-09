use std::sync::Arc;

use fuel_core::{
    combined_database::CombinedDatabase,
    database::{database_description::on_chain::OnChain, Database},
    state::{
        generic_database::GenericDatabase,
        iterable_key_value_view::IterableKeyValueViewWrapper,
    },
};
use fuel_core_bin::FuelService;
use fuel_core_importer::ports::ImporterDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::blockchain::consensus::Sealed;
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
    async fn start(&self) -> anyhow::Result<()>;
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

    fn get_latest_block_height(&self) -> anyhow::Result<u32> {
        Ok(self
            .onchain_database()
            .latest_block_height()?
            .map(|h| *h)
            .unwrap_or_default())
    }

    fn blocks_subscription(
        &self,
    ) -> Receiver<fuel_core_importer::ImporterResult>;

    fn get_sealed_block_by_height(&self, height: u32) -> Sealed<FuelCoreBlock> {
        self.onchain_database()
            .latest_view()
            .expect("failed to get latest db view")
            .get_sealed_block_by_height(&height.into())
            .expect("Failed to get latest block height")
            .expect("NATS Publisher: no block at height {height}")
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
    async fn start(&self) -> anyhow::Result<()> {
        fuel_core_bin::cli::init_logging();
        self.fuel_service.start_and_await().await?;
        Ok(())
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
}

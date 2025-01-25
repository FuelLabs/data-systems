use fuel_core::state::{
    generic_database::GenericDatabase,
    iterable_key_value_view::IterableKeyValueViewWrapper,
};
pub use fuel_core_client::client::{
    schema::Tai64Timestamp as FuelCoreTai64Timestamp,
    types::TransactionStatus as FuelCoreClientTransactionStatus,
};
pub use fuel_core_importer::ImporterResult as FuelCoreImporterResult;
pub use fuel_core_types::{
    blockchain::{
        block::Block as FuelCoreBlock,
        consensus::{
            poa::PoAConsensus as FuelCorePoAConsensus,
            Consensus as FuelCoreConsensus,
            Genesis as FuelCoreGenesis,
        },
        header::BlockHeader as FuelCoreBlockHeader,
        primitives::BlockId as FuelCoreBlockId,
        SealedBlock as FuelCoreSealedBlock,
    },
    fuel_asm::Word as FuelCoreWord,
    fuel_crypto::Signature as FuelCoreSignature,
    fuel_tx::{
        field::{Inputs as FuelCoreInputs, Outputs as FuelCoreOutputs},
        input::contract::Contract as FuelCoreInputContract,
        output::contract::Contract as FuelCoreOutputContract,
        policies::Policies as FuelCorePolicies,
        Address as FuelCoreAddress,
        AssetId as FuelCoreAssetId,
        BlobId as FuelCoreBlobId,
        Bytes32 as FuelCoreBytes32,
        Contract as FuelCoreContract,
        ContractId as FuelCoreContractId,
        Input as FuelCoreInput,
        MessageId as FuelCoreMessageId,
        Output as FuelCoreOutput,
        PanicInstruction as FuelCorePanicInstruction,
        Receipt as FuelCoreReceipt,
        ScriptExecutionResult as FuelCoreScriptExecutionResult,
        StorageSlot as FuelCoreStorageSlot,
        Transaction as FuelCoreTransaction,
        TxId as FuelCoreTxId,
        TxPointer as FuelCoreTxPointer,
        UniqueIdentifier as FuelCoreUniqueIdentifier,
        UpgradePurpose as FuelCoreUpgradePurpose,
        UtxoId as FuelCoreUtxoId,
    },
    fuel_types::{
        BlockHeight as FuelCoreBlockHeight,
        ChainId as FuelCoreChainId,
    },
    services::{
        block_importer::{
            ImportResult as FuelCoreImportResult,
            SharedImportResult as FuelCoreSharedImportResult,
        },
        txpool::TransactionStatus as FuelCoreTransactionStatus,
    },
    tai64::Tai64 as FuelCoreTai64,
};

pub type FuelCoreOffchainDatabase = GenericDatabase<
    IterableKeyValueViewWrapper<
        fuel_core::fuel_core_graphql_api::storage::Column,
    >,
>;

use std::sync::Arc;

use fuel_core::{
    combined_database::CombinedDatabase,
    database::{database_description::on_chain::OnChain, Database},
    fuel_core_graphql_api::ports::DatabaseBlocks,
};
use fuel_core_bin::FuelService;
use fuel_core_importer::ports::ImporterDatabase;
use fuel_core_storage::{
    tables::Transactions,
    transactional::AtomicView,
    StorageAsRef,
};
use fuel_core_types::blockchain::consensus::{Consensus, Sealed};
use tokio::sync::broadcast::Receiver;

#[derive(thiserror::Error, Debug)]
pub enum FuelCoreError {
    #[error("Failed to start Fuel Core: {0}")]
    Start(String),
    #[error("Failed to stop Fuel Core: {0}")]
    Stop(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Service error: {0}")]
    Service(String),
    #[error("Failed to await sync: {0}")]
    AwaitSync(String),
    #[error("Failed to sync offchain database: {0}")]
    OffchainSync(String),
    #[error("Failed to get block producer from consensus")]
    GetBlockProducer,
    #[error("Failed to get item from storage: {0}")]
    Storage(fuel_core_storage::Error),
    #[error("Failed to find transactions with tx_id: {0}")]
    TransactionNotFound(String),
}

pub type FuelCoreResult<T> = Result<T, FuelCoreError>;

/// Interface for `fuel-core` related logic.
/// This was introduced to simplify mocking and testing the `sv-publisher` crate.
#[async_trait::async_trait]
pub trait FuelCoreLike: Sync + Send {
    async fn start(&self) -> FuelCoreResult<()>;
    async fn stop(&self);

    fn is_started(&self) -> bool;
    fn base_asset_id(&self) -> &FuelCoreAssetId;
    fn chain_id(&self) -> &FuelCoreChainId;
    fn fuel_service(&self) -> &FuelService;
    fn database(&self) -> &CombinedDatabase;

    fn onchain_database(&self) -> &Database<OnChain> {
        self.database().on_chain()
    }

    fn offchain_database(
        &self,
    ) -> FuelCoreResult<Arc<FuelCoreOffchainDatabase>> {
        Ok(Arc::new(
            self.database()
                .off_chain()
                .latest_view()
                .map_err(|e| FuelCoreError::Database(e.to_string()))?,
        ))
    }

    fn blocks_subscription(
        &self,
    ) -> Receiver<fuel_core_importer::ImporterResult>;

    fn get_latest_block_height(&self) -> FuelCoreResult<crate::BlockHeight> {
        Ok(self
            .onchain_database()
            .latest_block_height()
            .map_err(|e| FuelCoreError::Database(e.to_string()))?
            .map(Into::into)
            .unwrap_or_default())
    }

    fn get_tx_by_id(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> FuelCoreResult<FuelCoreTransaction>;

    fn get_tx_status(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> FuelCoreResult<Option<FuelCoreTransactionStatus>>;

    fn get_receipts(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> FuelCoreResult<Option<Vec<FuelCoreReceipt>>>;

    fn get_consensus(
        &self,
        block_height: &FuelCoreBlockHeight,
    ) -> FuelCoreResult<Consensus> {
        self.onchain_database()
            .latest_view()
            .map_err(|e| FuelCoreError::Database(e.to_string()))?
            .consensus(block_height)
            .map_err(|e| FuelCoreError::Database(e.to_string()))
    }

    fn get_block_and_producer(
        &self,
        sealed_block: &Sealed<FuelCoreBlock>,
    ) -> FuelCoreResult<(FuelCoreBlock, crate::Address)> {
        let block = sealed_block.entity.clone();
        let block_producer = sealed_block
            .consensus
            .block_producer(&block.id())
            .map_err(|_| FuelCoreError::GetBlockProducer)?;
        Ok((block, block_producer.into()))
    }

    fn get_sealed_block_by_height(
        &self,
        block_height: crate::BlockHeight,
    ) -> Sealed<FuelCoreBlock> {
        let height = block_height.as_ref().to_owned() as u32;
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
        let snapshot = &fuel_service.shared.config.snapshot_reader;
        let chain_config = snapshot.chain_config();
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
    async fn start(&self) -> FuelCoreResult<()> {
        fuel_core_bin::cli::init_logging();
        self.fuel_service
            .start_and_await()
            .await
            .map_err(|e| FuelCoreError::Start(e.to_string()))?;
        Ok(())
    }

    fn is_started(&self) -> bool {
        self.fuel_service.state().started()
    }

    fn fuel_service(&self) -> &FuelService {
        &self.fuel_service
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

    fn get_tx_by_id(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> FuelCoreResult<FuelCoreTransaction> {
        let storage =
            self.database().on_chain().storage_as_ref::<Transactions>();
        let tx = storage.get(tx_id).map_err(FuelCoreError::Storage)?;
        match tx {
            Some(tx) => Ok(tx.into_owned()),
            None => Err(FuelCoreError::TransactionNotFound(tx_id.to_string())),
        }
    }

    fn get_tx_status(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> FuelCoreResult<Option<FuelCoreTransactionStatus>> {
        self.offchain_database()?
            .get_tx_status(tx_id)
            .map_err(|e| FuelCoreError::Database(e.to_string()))
    }

    fn get_receipts(
        &self,
        tx_id: &FuelCoreBytes32,
    ) -> FuelCoreResult<Option<Vec<FuelCoreReceipt>>> {
        let receipts = self
            .get_tx_status(tx_id)?
            .map(|status| match &status {
                FuelCoreTransactionStatus::Success { receipts, .. }
                | FuelCoreTransactionStatus::Failed { receipts, .. } => {
                    Some(receipts.clone())
                }
                _ => None,
            })
            .unwrap_or_default();
        Ok(receipts)
    }
}

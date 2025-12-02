pub use fuel_core::schema::tx::types::Transaction as FuelCoreTransaction;
pub use fuel_core_client::client::{
    schema::{
        tx::transparent_tx::{
            ConsensusParametersPurpose as FuelCoreClientConsensusParametersPurpose,
            Input as FuelCoreClientInput,
            Output as FuelCoreClientOutput,
            Policies as FuelCoreClientPolicies,
            StateTransitionPurpose as FuelCoreClientStateTransitionPurpose,
            Transaction as FuelCoreClientTransaction,
            UpgradePurpose as FuelCoreClientUpgradePurpose,
        },
        Tai64Timestamp as FuelCoreTai64Timestamp,
    },
    types::TransactionStatus as FuelCoreClientTransactionStatus,
};
pub use fuel_core_types::{
    blockchain::{
        block::Block as FuelCoreBlock,
        consensus::{
            poa::PoAConsensus as FuelCorePoAConsensus,
            Consensus as FuelCoreConsensus,
            Genesis as FuelCoreGenesis,
            Sealed as FuelCoreSealed,
        },
        header::BlockHeader as FuelCoreBlockHeader,
        primitives::{
            BlockId as FuelCoreBlockId,
            DaBlockHeight as FuelCoreDaBlockHeight,
        },
        SealedBlock as FuelCoreSealedBlock,
    },
    fuel_asm::Word as FuelCoreWord,
    fuel_crypto::Signature as FuelCoreSignature,
    fuel_tx::{
        field::{
            Inputs as FuelCoreInputs,
            Outputs as FuelCoreOutputs,
        },
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
        Transaction as FuelCoreTypesTransaction,
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
        executor::{
            Event as FuelCoreExecutorEvent,
            TransactionExecutionStatus as FuelCoreExecutorStatus,
        },
        transaction_status::{
            TransactionExecutionStatus as FuelCoreTransactionExecutionStatus,
            TransactionStatus as FuelCoreTransactionStatus,
        },
    },
    tai64::Tai64 as FuelCoreTai64,
};

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
    #[error("Failed to find transactions with tx_id: {0}")]
    TransactionNotFound(String),
}

pub type FuelCoreResult<T> = Result<T, FuelCoreError>;

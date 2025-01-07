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

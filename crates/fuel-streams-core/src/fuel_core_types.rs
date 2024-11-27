/// FuelCore Types
/// Allows flexilibity of aggregating and transforming them for different payload types
pub use fuel_core_types::fuel_tx::policies::Policies as FuelCorePolicies;
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
    },
    fuel_tx::{
        field::{Inputs as FuelCoreInputs, Outputs as FuelCoreOutputs},
        input::contract::Contract as FuelCoreInputContract,
        output::contract::Contract as FuelCoreOutputContract,
        Address as FuelCoreAddress,
        AssetId as FuelCoreAssetId,
        BlobId as FuelCoreBlobId,
        Bytes32 as FuelCoreBytes32,
        Contract as FuelCoreContract,
        ContractId as FuelCoreContractId,
        Input as FuelCoreInput,
        MessageId as FuelCoreMessageId,
        Output as FuelCoreOutput,
        Receipt as FuelCoreReceipt,
        Transaction as FuelCoreTransaction,
        TxPointer as FuelCoreTxPointer,
        UniqueIdentifier as FuelCoreUniqueIdentifier,
        UpgradePurpose as FuelCoreUpgradePurpose,
        UtxoId as FuelCoreUtxoId,
        Word as FuelCoreWord,
    },
    fuel_types::{
        BlockHeight as FuelCoreBlockHeight,
        ChainId as FuelCoreChainId,
    },
    services::{
        block_importer::ImportResult as FuelCoreImportResult,
        txpool::TransactionStatus as FuelCoreTransactionStatus,
    },
    tai64::Tai64 as FuelCoreTai64,
};

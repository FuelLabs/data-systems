/// FuelCore Types
/// Allows the flexilibity of aggregating the FuelCore types in any payload
pub use fuel_core_types::fuel_tx::policies::Policies as FuelCorePolicies;
pub use fuel_core_types::{
    blockchain::{
        block::Block as FuelCoreBlock,
        consensus::{
            poa::PoAConsensus,
            Consensus as FuelCoreConsensus,
            Genesis,
        },
        header::BlockHeader,
    },
    fuel_tx::{
        field::{Inputs, Outputs},
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
        TxPointer,
        UniqueIdentifier,
        UpgradePurpose as FuelCoreUpgradePurpose,
        UtxoId,
        Word,
    },
    fuel_types::{BlockHeight as FuelCoreBlockHeight, ChainId},
    services::{
        block_importer::ImportResult,
        txpool::TransactionStatus as FuelCoreTransactionStatus,
    },
    tai64::Tai64,
};

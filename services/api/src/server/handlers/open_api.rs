use fuel_streams_core::types::{
    Amount,
    BlobId,
    BlockHeader,
    BlockId,
    BlockVersion,
    BurnReceipt,
    CallReceipt,
    Consensus,
    FuelCoreUpgradePurposeWrapper,
    GasAmount,
    HexData,
    Input,
    InputCoin,
    InputContract,
    InputMessage,
    LogDataReceipt,
    LogReceipt,
    MessageOutReceipt,
    MintReceipt,
    Nonce,
    Output,
    OutputChange,
    OutputCoin,
    OutputContract,
    OutputContractCreated,
    OutputVariable,
    PanicReceipt,
    PolicyWrapper,
    Receipt,
    ReturnDataReceipt,
    ReturnReceipt,
    RevertReceipt,
    Salt,
    ScriptResultReceipt,
    StorageSlot,
    TransferOutReceipt,
    TransferReceipt,
    TxPointer,
    UtxoId,
};
use fuel_streams_domains::{
    blocks::queryable::BlocksQuery,
    inputs::queryable::InputsQuery,
    outputs::queryable::OutputsQuery,
    queryable::ValidatedQuery,
    receipts::queryable::ReceiptsQuery,
    transactions::queryable::TransactionsQuery,
};
use utoipa::OpenApi;

use super::{
    accounts::*,
    blocks::*,
    contracts::*,
    inputs::*,
    outputs::*,
    receipts::*,
    transactions::*,
    utxos::*,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_blocks,
        get_block_transactions,
        get_block_receipts,
        get_block_inputs,
        get_block_outputs,
        get_accounts_transactions,
        get_accounts_inputs,
        get_accounts_outputs,
        get_accounts_utxos,
        get_contracts_transactions,
        get_contracts_inputs,
        get_contracts_outputs,
        get_contracts_utxos,
        get_inputs,
        get_outputs,
        get_receipts,
        get_transactions,
        get_transaction_receipts,
        get_transaction_inputs,
        get_transaction_outputs,
        get_utxos,
    ),
    components(schemas(
        ValidatedQuery<BlocksQuery>,
        ValidatedQuery<TransactionsQuery>,
        ValidatedQuery<ReceiptsQuery>,
        ValidatedQuery<InputsQuery>,
        ValidatedQuery<OutputsQuery>,
        Consensus,
        BlockHeader,
        BlockId,
        BlockVersion,
        InputContract,
        InputCoin,
        InputMessage,
        OutputCoin,
        OutputContract,
        OutputChange,
        OutputVariable,
        OutputContractCreated,
        BlobId,
        Input,
        Amount,
        Output,
        PolicyWrapper,
        HexData,
        Receipt,
        Salt,
        GasAmount,
        StorageSlot,
        TxPointer,
        FuelCoreUpgradePurposeWrapper,
        CallReceipt,
        ReturnReceipt,
        ReturnDataReceipt,
        PanicReceipt,
        RevertReceipt,
        LogReceipt,
        LogDataReceipt,
        TransferReceipt,
        TransferOutReceipt,
        ScriptResultReceipt,
        MessageOutReceipt,
        MintReceipt,
        BurnReceipt,
        Nonce,
        UtxoId,
    )),
    tags(
        (name = "Blocks", description = "Block retrieval endpoints"),
        (name = "Accounts", description = "Accounts retrieval endpoints"),
        (name = "Contracts", description = "Contracts retrieval endpoints"),
        (name = "Inputs", description = "Inputs retrieval endpoints"),
        (name = "Outputs", description = "Outputs retrieval endpoints"),
        (name = "Receipts", description = "Receipts retrieval endpoints"),
        (name = "Transactions", description = "Transactions retrieval endpoints"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub struct ApiDoc;

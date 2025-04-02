#![allow(clippy::disallowed_methods)]

use fuel_streams_core::types::{
    Amount,
    BlobId,
    BlockHeader,
    BlockId,
    BlockVersion,
    BurnReceipt,
    CallReceipt,
    Consensus,
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
    UpgradePurpose,
    UtxoId,
};
use fuel_streams_domains::{
    blocks::BlocksQuery,
    inputs::InputsQuery,
    outputs::OutputsQuery,
    receipts::ReceiptsQuery,
    transactions::TransactionsQuery,
};
use fuel_web_utils::{api_key::ApiKey, server::server_builder::API_BASE_PATH};
use utoipa::{
    openapi::{
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        Server,
    },
    Modify,
    OpenApi,
};

pub const TAG_ACCOUNTS: &str = "Accounts";
pub const TAG_BLOCKS: &str = "Blocks";
pub const TAG_CONTRACTS: &str = "Contracts";
pub const TAG_INPUTS: &str = "Inputs";
pub const TAG_OUTPUTS: &str = "Outputs";
pub const TAG_RECEIPTS: &str = "Receipts";
pub const TAG_TRANSACTIONS: &str = "Transactions";
pub const TAG_UTXOS: &str = "Utxos";
pub const TAG_PREDICATES: &str = "Predicates";
pub const TAG_API_KEYS: &str = "ApiKeys";

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().add_security_scheme(
            "api_key",
            SecurityScheme::Http(
                HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build(),
            ),
        );
    }
}

struct ServerAddon;

impl Modify for ServerAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new(API_BASE_PATH)])
    }
}

use super::{
    accounts::*,
    api_key::*,
    blocks::*,
    contracts::*,
    inputs::*,
    outputs::*,
    predicates::*,
    receipts::*,
    transactions::*,
    utxos::*,
};
use crate::server::handlers::api_key::GenerateApiKeyRequest;

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
        get_inputs_message,
        get_inputs_contract,
        get_inputs_coin,
        get_outputs,
        get_outputs_coin,
        get_outputs_contract,
        get_outputs_change,
        get_outputs_variable,
        get_receipts,
        get_receipts_call,
        get_receipts_return,
        get_receipts_return_data,
        get_receipts_panic,
        get_receipts_revert,
        get_receipts_log,
        get_receipts_log_data,
        get_receipts_transfer,
        get_receipts_transfer_out,
        get_receipts_script_result,
        get_receipts_message_out,
        get_receipts_mint,
        get_receipts_burn,
        get_transactions,
        get_transaction_receipts,
        get_transaction_inputs,
        get_transaction_outputs,
        get_utxos,
        get_predicates,
        generate_api_key,
    ),
    components(schemas(
        BlocksQuery,
        TransactionsQuery,
        ReceiptsQuery,
        InputsQuery,
        OutputsQuery,
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
        UpgradePurpose,
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
        GenerateApiKeyRequest,
        ApiKey,
    )),
    tags(
        (name = "Blocks", description = "Block retrieval endpoints"),
        (name = "Accounts", description = "Accounts retrieval endpoints"),
        (name = "Contracts", description = "Contracts retrieval endpoints"),
        (name = "Inputs", description = "Inputs retrieval endpoints"),
        (name = "Outputs", description = "Outputs retrieval endpoints"),
        (name = "Receipts", description = "Receipts retrieval endpoints"),
        (name = "Transactions", description = "Transactions retrieval endpoints"),
        (name = "Predicates", description = "Predicates retrieval endpoints"),
        (name = "ApiKeys", description = "Api Key generation"),
    ),
    modifiers(&SecurityAddon, &ServerAddon)
)]
pub struct ApiDoc;

use axum::{middleware::from_fn_with_state, routing::get, Router};
use fuel_streams_core::types::{
    InputType,
    OutputType,
    ReceiptType,
    StreamResponse,
};
use fuel_streams_store::{db::DbItem, record::RecordPointer};
use fuel_web_utils::{
    api_key::middleware::ApiKeyAuth,
    router_builder::RouterBuilder,
};
use serde::Serialize;

use super::{errors::ApiError, handlers::*, state::ServerState};

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetDataResponse {
    data: Vec<StreamResponse>,
}

impl<T> TryFrom<Vec<T>> for GetDataResponse
where
    T: DbItem + Into<RecordPointer>,
{
    type Error = ApiError;
    fn try_from(items: Vec<T>) -> Result<Self, Self::Error> {
        let data = items
            .into_iter()
            .map(|item| {
                StreamResponse::try_from((item.subject_id(), item))
                    .map_err(ApiError::Stream)
            })
            .collect::<Result<Vec<StreamResponse>, ApiError>>();
        data.map(|collected_data| GetDataResponse {
            data: collected_data,
        })
    }
}

pub fn create_routes(state: &ServerState) -> Router {
    let app = Router::new();
    let api_key_middleware =
        ApiKeyAuth::new(&state.api_keys_manager, &state.db);

    let (blocks_path, blocks_router) = RouterBuilder::new("blocks")
        .root(get(blocks::get_blocks))
        .related(":height/receipts", get(blocks::get_block_receipts))
        .related(":height/transactions", get(blocks::get_block_transactions))
        .related(":height/inputs", get(blocks::get_block_inputs))
        .related(":height/outputs", get(blocks::get_block_outputs))
        .build();

    let (accounts_path, accounts_router) = RouterBuilder::new("accounts")
        .related(
            ":address/transactions",
            get(accounts::get_accounts_transactions),
        )
        .related(":address/inputs", get(accounts::get_accounts_inputs))
        .related(":address/outputs", get(accounts::get_accounts_outputs))
        .related(":address/utxos", get(accounts::get_accounts_utxos))
        .build();

    let (contracts_path, contracts_router) = RouterBuilder::new("contracts")
        .related(
            ":contractId/transactions",
            get(contracts::get_contracts_transactions),
        )
        .related(":contractId/inputs", get(contracts::get_contracts_inputs))
        .related(":contractId/outputs", get(contracts::get_contracts_outputs))
        .related(":contractId/utxos", get(contracts::get_contracts_utxos))
        .build();

    let (inputs_path, inputs_router) = RouterBuilder::new("inputs")
        .root(get(inputs::get_inputs))
        .typed_routes(
            &[
                InputType::Message.as_str(),
                InputType::Coin.as_str(),
                InputType::Contract.as_str(),
            ],
            get(inputs::get_inputs),
        )
        .build();

    let (outputs_path, outputs_router) = RouterBuilder::new("outputs")
        .root(get(outputs::get_outputs))
        .typed_routes(
            &[
                OutputType::Coin.as_str(),
                OutputType::Contract.as_str(),
                OutputType::Change.as_str(),
                OutputType::Variable.as_str(),
                OutputType::ContractCreated.as_str(),
            ],
            get(outputs::get_outputs),
        )
        .build();

    let (receipts_path, receipts_router) = RouterBuilder::new("receipts")
        .root(get(receipts::get_receipts))
        .typed_routes(
            &[
                ReceiptType::Call.as_str(),
                ReceiptType::Return.as_str(),
                ReceiptType::ReturnData.as_str(),
                ReceiptType::Panic.as_str(),
                ReceiptType::Revert.as_str(),
                ReceiptType::Log.as_str(),
                ReceiptType::LogData.as_str(),
                ReceiptType::Transfer.as_str(),
                ReceiptType::TransferOut.as_str(),
                ReceiptType::ScriptResult.as_str(),
                ReceiptType::MessageOut.as_str(),
                ReceiptType::Mint.as_str(),
                ReceiptType::Burn.as_str(),
            ],
            get(receipts::get_receipts),
        )
        .build();

    let (transactions_path, transactions_router) =
        RouterBuilder::new("transactions")
            .root(get(transactions::get_transactions))
            .related(
                ":txId/receipts",
                get(transactions::get_transaction_receipts),
            )
            .related(":txId/inputs", get(transactions::get_transaction_inputs))
            .related(
                ":txId/outputs",
                get(transactions::get_transaction_outputs),
            )
            .build();

    let (utxos_path, utxos_router) = RouterBuilder::new("utxos")
        .root(get(utxos::get_utxos))
        .build();

    app.nest(&blocks_path, blocks_router)
        .nest(&accounts_path, accounts_router)
        .nest(&contracts_path, contracts_router)
        .nest(&inputs_path, inputs_router)
        .nest(&outputs_path, outputs_router)
        .nest(&receipts_path, receipts_router)
        .nest(&transactions_path, transactions_router)
        .nest(&utxos_path, utxos_router)
        .layer(from_fn_with_state(
            api_key_middleware.clone(),
            ApiKeyAuth::middleware,
        ))
        .with_state(state.clone())
}

pub mod accounts;
pub mod blocks;
pub mod contracts;
pub mod inputs;
pub mod macros;
pub mod outputs;
pub mod receipts;
pub mod transactions;
pub mod utxos;
use actix_web::{http::StatusCode, web};
use fuel_streams_core::types::{StreamResponse, StreamResponseError};
use fuel_streams_domains::{
    inputs::InputType,
    outputs::OutputType,
    receipts::ReceiptType,
};
use fuel_streams_store::{db::DbItem, record::RecordPointer};
use fuel_web_utils::{
    api_key::middleware::ApiKeyAuth,
    server::api::with_prefixed_route,
};
use serde::Serialize;

use super::handlers;
use crate::{
    related_resource_endpoint,
    resource_with_related_endpoints,
    server::state::ServerState,
    simple_resource_endpoint,
    typed_resource_endpoint,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Validation error {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Stream response error {0}")]
    Stream(#[from] StreamResponseError),
}

impl From<Error> for actix_web::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Sqlx(e) => actix_web::error::InternalError::new(
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into(),
            Error::Validation(e) => {
                actix_web::error::InternalError::new(e, StatusCode::BAD_REQUEST)
                    .into()
            }
            Error::Stream(e) => actix_web::error::InternalError::new(
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDataResponse {
    data: Vec<StreamResponse>,
}

impl<T> TryFrom<Vec<T>> for GetDataResponse
where
    T: DbItem + Into<RecordPointer>,
{
    type Error = Error;
    fn try_from(items: Vec<T>) -> Result<Self, Self::Error> {
        let data = items
            .into_iter()
            .map(|item| {
                StreamResponse::try_from((item.subject_id(), item))
                    .map_err(Error::Stream)
            })
            .collect::<Result<Vec<StreamResponse>, Error>>();
        data.map(|collected_data| GetDataResponse {
            data: collected_data,
        })
    }
}

pub fn create_services(
    state: ServerState,
) -> impl Fn(&mut web::ServiceConfig) + Send + Sync + 'static {
    move |cfg: &mut web::ServiceConfig| {
        let api_key_middleware =
            ApiKeyAuth::new(&state.api_keys_manager, &state.db);
        cfg.app_data(web::Data::new(state.clone()));

        // blocks
        resource_with_related_endpoints!(
            cfg,
            api_key_middleware,
            "blocks",
            "height",
            handlers::blocks::get_blocks,
            [
                ("receipts", handlers::blocks::get_block_receipts),
                ("transactions", handlers::blocks::get_block_transactions),
                ("inputs", handlers::blocks::get_block_inputs),
                ("outputs", handlers::blocks::get_block_outputs)
            ]
        );

        // transactions
        resource_with_related_endpoints!(
            cfg,
            api_key_middleware,
            "transactions",
            "tx_id",
            handlers::transactions::get_transactions,
            [
                ("receipts", handlers::transactions::get_transaction_receipts),
                ("inputs", handlers::transactions::get_transaction_inputs),
                ("outputs", handlers::transactions::get_transaction_outputs)
            ]
        );

        // inputs
        typed_resource_endpoint!(
            cfg,
            api_key_middleware,
            "inputs",
            handlers::inputs::get_inputs,
            InputType,
            ("/message", Message),
            ("/contract", Contract),
            ("/coin", Coin)
        );

        // outputs
        typed_resource_endpoint!(
            cfg,
            api_key_middleware,
            "outputs",
            handlers::outputs::get_outputs,
            OutputType,
            ("/change", Change),
            ("/coin", Coin),
            ("/contract", Contract),
            ("/contract-created", ContractCreated),
            ("/variable", Variable)
        );

        // utxos
        simple_resource_endpoint!(
            cfg,
            api_key_middleware,
            "utxos",
            handlers::utxos::get_utxos
        );

        // receipts
        typed_resource_endpoint!(
            cfg,
            api_key_middleware,
            "receipts",
            handlers::receipts::get_receipts,
            ReceiptType,
            ("/burn", Burn),
            ("/mint", Mint),
            ("/message-out", MessageOut),
            ("/script-result", ScriptResult),
            ("/transfer-out", TransferOut),
            ("/transfer", Transfer),
            ("/logdata", LogData),
            ("/log", Log),
            ("/revert", Revert),
            ("/panic", Panic),
            ("/return-data", ReturnData),
            ("/return", Return),
            ("/call", Call)
        );

        // contracts
        related_resource_endpoint!(
            cfg,
            api_key_middleware,
            "contracts",
            "contract_id",
            [
                // ( // TODO: need to extend db with normalized values
                //     "transactions",
                //     handlers::contracts::get_contracts_transactions
                // ),
                ("inputs", handlers::contracts::get_contracts_inputs),
                ("outputs", handlers::contracts::get_contracts_outputs),
                ("utxos", handlers::contracts::get_contracts_utxos)
            ]
        );

        // accounts
        related_resource_endpoint!(
            cfg,
            api_key_middleware,
            "accounts",
            "address",
            [
                // ( // TODO: need to extend db with normalized values
                //     "transactions",
                //     handlers::accounts::get_accounts_transactions
                // ),
                ("inputs", handlers::accounts::get_accounts_inputs),
                ("outputs", handlers::accounts::get_accounts_outputs),
                ("utxos", handlers::accounts::get_accounts_utxos)
            ]
        );
    }
}

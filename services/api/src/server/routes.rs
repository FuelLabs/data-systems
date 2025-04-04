use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use fuel_streams_core::types::StreamResponse;
use fuel_streams_domains::infra::{db::DbItem, record::RecordPointer};
use fuel_web_utils::{
    api_key::middleware::ApiKeyMiddleware,
    router_builder::RouterBuilder,
    server::server_builder::API_BASE_PATH,
};
use open_api::ApiDoc;
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::{
    errors::ApiError,
    handlers::*,
    middleware::validate_scope_middleware,
    state::ServerState,
};

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
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
    let app = Router::new().merge(
        SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()),
    );

    let manager = state.api_keys_manager.clone();
    let db = state.db.clone();
    let auth_params = (manager.clone(), db.clone());

    let (key_path, key_router) = RouterBuilder::new("/keys")
        .related("/generate", post(api_key::generate_api_key))
        .with_layer(from_fn(api_key::validate_manage_api_keys_scope))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (blocks_path, blocks_router) = RouterBuilder::new("/blocks")
        .root(get(blocks::get_blocks))
        .related("/{height}/receipts", get(blocks::get_block_receipts))
        .related(
            "/{height}/transactions",
            get(blocks::get_block_transactions),
        )
        .related("/{height}/inputs", get(blocks::get_block_inputs))
        .related("/{height}/outputs", get(blocks::get_block_outputs))
        .with_layer(from_fn(validate_scope_middleware))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (accounts_path, accounts_router) = RouterBuilder::new("/accounts")
        .related(
            "/{address}/transactions",
            get(accounts::get_accounts_transactions),
        )
        .related("/{address}/inputs", get(accounts::get_accounts_inputs))
        .related("/{address}/outputs", get(accounts::get_accounts_outputs))
        .related("/{address}/utxos", get(accounts::get_accounts_utxos))
        .related("/{address}/receipts", get(accounts::get_accounts_receipts))
        .with_layer(from_fn(validate_scope_middleware))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (contracts_path, contracts_router) = RouterBuilder::new("/contracts")
        .related(
            "/{contract_id}/transactions",
            get(contracts::get_contracts_transactions),
        )
        .related(
            "/{contract_id}/inputs",
            get(contracts::get_contracts_inputs),
        )
        .related(
            "/{contract_id}/outputs",
            get(contracts::get_contracts_outputs),
        )
        .related("/{contract_id}/utxos", get(contracts::get_contracts_utxos))
        .related(
            "/{contract_id}/receipts",
            get(contracts::get_contracts_receipts),
        )
        .with_layer(from_fn(validate_scope_middleware))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (inputs_path, inputs_router) = RouterBuilder::new("/inputs")
        .root(get(inputs::get_inputs))
        .related("/message", get(inputs::get_inputs_message))
        .related("/contract", get(inputs::get_inputs_contract))
        .related("/coin", get(inputs::get_inputs_coin))
        .with_layer(from_fn(validate_scope_middleware))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (outputs_path, outputs_router) = RouterBuilder::new("/outputs")
        .root(get(outputs::get_outputs))
        .related("/coin", get(outputs::get_outputs_coin))
        .related("/contract", get(outputs::get_outputs_contract))
        .related("/change", get(outputs::get_outputs_change))
        .related("/variable", get(outputs::get_outputs_variable))
        .related(
            "/contract_created",
            get(outputs::get_outputs_contract_created),
        )
        .with_layer(from_fn(validate_scope_middleware))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (receipts_path, receipts_router) = RouterBuilder::new("/receipts")
        .root(get(receipts::get_receipts))
        .related("/call", get(receipts::get_receipts_call))
        .related("/return", get(receipts::get_receipts_return))
        .related("/return_data", get(receipts::get_receipts_return_data))
        .related("/panic", get(receipts::get_receipts_panic))
        .related("/revert", get(receipts::get_receipts_revert))
        .related("/log", get(receipts::get_receipts_log))
        .related("/log_data", get(receipts::get_receipts_log_data))
        .related("/transfer", get(receipts::get_receipts_transfer))
        .related("/transfer_out", get(receipts::get_receipts_transfer_out))
        .related("/script_result", get(receipts::get_receipts_script_result))
        .related("/message_out", get(receipts::get_receipts_message_out))
        .related("/mint", get(receipts::get_receipts_mint))
        .related("/burn", get(receipts::get_receipts_burn))
        .with_layer(from_fn(validate_scope_middleware))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (transactions_path, transactions_router) =
        RouterBuilder::new("/transactions")
            .root(get(transactions::get_transactions))
            .related(
                "/{tx_id}/receipts",
                get(transactions::get_transaction_receipts),
            )
            .related(
                "/{tx_id}/inputs",
                get(transactions::get_transaction_inputs),
            )
            .related(
                "/{tx_id}/outputs",
                get(transactions::get_transaction_outputs),
            )
            .related("/{tx_id}/utxos", get(transactions::get_transaction_utxos))
            .with_layer(from_fn(validate_scope_middleware))
            .with_layer(from_fn_with_state(
                auth_params.clone(),
                ApiKeyMiddleware::handler,
            ))
            .build();

    let (utxos_path, utxos_router) = RouterBuilder::new("/utxos")
        .root(get(utxos::get_utxos))
        .with_layer(from_fn(validate_scope_middleware))
        .with_layer(from_fn_with_state(
            auth_params.clone(),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (predicates_path, predicates_router) =
        RouterBuilder::new("/predicates")
            .root(get(predicates::get_predicates))
            .with_layer(from_fn(validate_scope_middleware))
            .with_layer(from_fn_with_state(
                auth_params.clone(),
                ApiKeyMiddleware::handler,
            ))
            .build();

    let routes = Router::new()
        .nest(&key_path, key_router)
        .nest(&blocks_path, blocks_router)
        .nest(&accounts_path, accounts_router)
        .nest(&contracts_path, contracts_router)
        .nest(&inputs_path, inputs_router)
        .nest(&outputs_path, outputs_router)
        .nest(&receipts_path, receipts_router)
        .nest(&transactions_path, transactions_router)
        .nest(&utxos_path, utxos_router)
        .nest(&predicates_path, predicates_router);

    app.nest(API_BASE_PATH, routes).with_state(state.clone())
}

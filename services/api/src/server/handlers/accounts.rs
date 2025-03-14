use axum::{
    extract::{FromRequest, Path, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::{
    Address,
    AssetId,
    BlockHeight,
    ContractId,
    HexData,
    InputType,
    OutputType,
    TransactionStatus,
    TransactionType,
    TxId,
};
use fuel_streams_domains::{
    inputs::queryable::InputsQuery,
    outputs::queryable::OutputsQuery,
    queryable::{Queryable, ValidatedQuery},
    transactions::queryable::TransactionsQuery,
    utxos::queryable::UtxosQuery,
};
use fuel_web_utils::api_key::ApiKey;

use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/accounts/{address}/transactions",
    tag = "accounts",
    params(
        ("address" = String, Path, description = "Account address"),
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("txStatus" = Option<TransactionStatus>, Query, description = "Filter by transaction status"),
        ("type" = Option<TransactionType>, Query, description = "Filter by transaction type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("after" = Option<i32>, Query, description = "Return transactions after this height"),
        ("before" = Option<i32>, Query, description = "Return transactions before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved account transactions", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Account not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_accounts_transactions(
    State(state): State<ServerState>,
    Path(address): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query =
        ValidatedQuery::<TransactionsQuery>::from_request(req, &state)
            .await?
            .into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(ApiError::Sqlx)?
        .try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/accounts/{address}/inputs",
    tag = "accounts",
    params(
        ("address" = String, Path, description = "Account address"),
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("inputType" = Option<InputType>, Query, description = "Filter by input type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("ownerId" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract inputs)"),
        ("senderAddress" = Option<Address>, Query, description = "Filter by sender address (for message inputs)"),
        ("recipientAddress" = Option<Address>, Query, description = "Filter by recipient address (for message inputs)"),
        ("after" = Option<i32>, Query, description = "Return inputs after this height"),
        ("before" = Option<i32>, Query, description = "Return inputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved account inputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Account not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_accounts_inputs(
    State(state): State<ServerState>,
    Path(address): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = ValidatedQuery::<InputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(ApiError::Sqlx)?
        .try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/accounts/{address}/outputs",
    tag = "accounts",
    params(
        ("address" = String, Path, description = "Account address"),
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("outputIndex" = Option<i32>, Query, description = "Filter by output index"),
        ("outputType" = Option<OutputType>, Query, description = "Filter by output type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("toAddress" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
        ("after" = Option<i32>, Query, description = "Return outputs after this height"),
        ("before" = Option<i32>, Query, description = "Return outputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved account outputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Account not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_accounts_outputs(
    State(state): State<ServerState>,
    Path(address): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = ValidatedQuery::<OutputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(ApiError::Sqlx)?
        .try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/accounts/{address}/utxos",
    tag = "accounts",
    params(
        ("address" = String, Path, description = "Account address"),
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("utxoType" = Option<InputType>, Query, description = "Filter by UTXO type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("utxoId" = Option<HexData>, Query, description = "Filter by UTXO ID"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract UTXOs)"),
        ("after" = Option<i32>, Query, description = "Return UTXOs after this height"),
        ("before" = Option<i32>, Query, description = "Return UTXOs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved account UTXOs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Account not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_accounts_utxos(
    State(state): State<ServerState>,
    Path(address): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = ValidatedQuery::<UtxosQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(ApiError::Sqlx)?
        .try_into()?;
    Ok(Json(response))
}

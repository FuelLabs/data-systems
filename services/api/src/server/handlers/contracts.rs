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

use super::open_api::TAG_CONTRACTS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/transactions",
    tag = TAG_CONTRACTS,
    params(
        ("contractId" = String, Path, description = "Contract ID"),
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
        (status = 200, description = "Successfully retrieved contract transactions", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_transactions(
    State(state): State<ServerState>,
    Path(contract_id): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query =
        ValidatedQuery::<TransactionsQuery>::from_request(req, &state)
            .await?
            .into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse =
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/inputs",
    tag = TAG_CONTRACTS,
    params(
        ("contractId" = String, Path, description = "Contract ID"),
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("inputType" = Option<InputType>, Query, description = "Filter by input type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("ownerId" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
        ("senderAddress" = Option<Address>, Query, description = "Filter by sender address (for message inputs)"),
        ("recipientAddress" = Option<Address>, Query, description = "Filter by recipient address (for message inputs)"),
        ("after" = Option<i32>, Query, description = "Return inputs after this height"),
        ("before" = Option<i32>, Query, description = "Return inputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract inputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_inputs(
    State(state): State<ServerState>,
    Path(contract_id): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<InputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse =
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/outputs",
    tag = TAG_CONTRACTS,
    params(
        ("contractId" = String, Path, description = "Contract ID"),
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("outputIndex" = Option<i32>, Query, description = "Filter by output index"),
        ("outputType" = Option<OutputType>, Query, description = "Filter by output type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("toAddress" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
        ("after" = Option<i32>, Query, description = "Return outputs after this height"),
        ("before" = Option<i32>, Query, description = "Return outputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract outputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_outputs(
    State(state): State<ServerState>,
    Path(contract_id): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<OutputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse =
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/utxos",
    tag = TAG_CONTRACTS,
    params(
        ("contractId" = String, Path, description = "Contract ID"),
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("utxoType" = Option<InputType>, Query, description = "Filter by UTXO type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("utxoId" = Option<HexData>, Query, description = "Filter by UTXO ID"),
        ("after" = Option<i32>, Query, description = "Return UTXOs after this height"),
        ("before" = Option<i32>, Query, description = "Return UTXOs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract UTXOs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_utxos(
    State(state): State<ServerState>,
    Path(contract_id): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<UtxosQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse =
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}

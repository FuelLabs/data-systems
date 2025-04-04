use std::str::FromStr;

use axum::{
    extract::{FromRequest, Path, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    infra::{
        repository::{Repository, ValidatedQuery},
        Cursor,
        OrderBy,
        TimeRange,
    },
    inputs::InputsQuery,
    outputs::OutputsQuery,
    receipts::ReceiptsQuery,
    transactions::TransactionsQuery,
    utxos::UtxosQuery,
};

use super::open_api::TAG_CONTRACTS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/contracts/{contract_id}/transactions",
    tag = TAG_CONTRACTS,
    params(
        ("contract_id" = String, Path, description = "Contract ID"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("tx_status" = Option<TransactionStatus>, Query, description = "Filter by transaction status"),
        ("type" = Option<TransactionType>, Query, description = "Filter by transaction type"),
        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("blob_id" = Option<BlobId>, Query, description = "Filter by blob ID"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return transactions after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return transactions before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
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
        Transaction::find_many(&state.db.pool, &query)
            .await?
            .try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contract_id}/inputs",
    tag = TAG_CONTRACTS,
    params(
        ("contract_id" = String, Path, description = "Contract ID"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("input_index" = Option<i32>, Query, description = "Filter by input index"),
        ("input_type" = Option<InputType>, Query, description = "Filter by input type"),
        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("owner_id" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
        ("asset_id" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
        ("sender_address" = Option<Address>, Query, description = "Filter by sender address (for message inputs)"),
        ("recipient_address" = Option<Address>, Query, description = "Filter by recipient address (for message inputs)"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return inputs after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return inputs before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
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
        Input::find_many(&state.db.pool, &query).await?.try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contract_id}/outputs",
    tag = TAG_CONTRACTS,
    params(
        ("contract_id" = String, Path, description = "Contract ID"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("output_index" = Option<i32>, Query, description = "Filter by output index"),
        ("output_type" = Option<OutputType>, Query, description = "Filter by output type"),
        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("to_address" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
        ("asset_id" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return outputs after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return outputs before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
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
    let response: GetDataResponse = Output::find_many(&state.db.pool, &query)
        .await?
        .try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contract_id}/utxos",
    tag = TAG_CONTRACTS,
    params(
        ("contract_id" = String, Path, description = "Contract ID"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("input_index" = Option<i32>, Query, description = "Filter by input index"),
        ("output_index" = Option<i32>, Query, description = "Filter by output index"),
        ("type" = Option<UtxoType>, Query, description = "Filter by UTXO type"),
        ("status" = Option<UtxoStatus>, Query, description = "Filter by UTXO status"),
        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("utxo_id" = Option<UtxoId>, Query, description = "Filter by UTXO ID"),
        ("from" = Option<Address>, Query, description = "Filter by source address"),
        ("to" = Option<Address>, Query, description = "Filter by destination address"),
        ("asset_id" = Option<AssetId>, Query, description = "Filter by asset ID"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return UTXOs after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return UTXOs before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
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
        Utxo::find_many(&state.db.pool, &query).await?.try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contract_id}/receipts",
    tag = TAG_CONTRACTS,
    params(
        ("contract_id" = String, Path, description = "Contract ID"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("receipt_index" = Option<i32>, Query, description = "Filter by receipt index"),
        ("receipt_type" = Option<ReceiptType>, Query, description = "Filter by receipt type"),
        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
        ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
        ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
        ("sender" = Option<Address>, Query, description = "Filter by sender address"),
        ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
        ("sub_id" = Option<Bytes32>, Query, description = "Filter by sub ID"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return receipts after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return receipts before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract receipts", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_receipts(
    State(state): State<ServerState>,
    Path(contract_id): Path<String>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<ReceiptsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_contract(
        &ContractId::from_str(&contract_id)
            .map_err(ApiError::InvalidContractId)?,
    );
    let response: GetDataResponse = Receipt::find_many(&state.db.pool, &query)
        .await?
        .try_into()?;
    Ok(Json(response))
}

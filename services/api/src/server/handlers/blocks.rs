use axum::{
    extract::{FromRequest, Path, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    blocks::BlocksQuery,
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
};

use super::open_api::TAG_BLOCKS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/blocks",
    tag = TAG_BLOCKS,
    params(
        ("producer" = Option<Address>, Query, description = "Filter by block producer address"),
        ("height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return blocks after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return blocks before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved blocks", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_blocks(
    State(state): State<ServerState>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let query = ValidatedQuery::<BlocksQuery>::from_request(req, &state)
        .await?
        .into_inner();
    let response: GetDataResponse =
        Block::find_many(&state.db.pool, &query).await?.try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{height}/transactions",
    tag = TAG_BLOCKS,
    params(
        ("height" = BlockHeight, Path, description = "Block height"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("tx_status" = Option<TransactionStatus>, Query, description = "Filter by transaction status"),
        ("type" = Option<TransactionType>, Query, description = "Filter by transaction type"),
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
        (status = 200, description = "Successfully retrieved block transactions", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Block not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_block_transactions(
    State(state): State<ServerState>,
    Path(height): Path<u64>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query =
        ValidatedQuery::<TransactionsQuery>::from_request(req, &state)
            .await?
            .into_inner();
    let block_height = height;
    query.set_block_height(block_height);
    let response: GetDataResponse =
        Transaction::find_many(&state.db.pool, &query)
            .await?
            .try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{height}/receipts",
    tag = TAG_BLOCKS,
    params(
        ("height" = BlockHeight, Path, description = "Block height"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("receipt_index" = Option<i32>, Query, description = "Filter by receipt index"),
        ("receipt_type" = Option<ReceiptType>, Query, description = "Filter by receipt type"),
        ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
        ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
        ("contract" = Option<ContractId>, Query, description = "Filter by contract ID"),
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
        (status = 200, description = "Successfully retrieved block receipts", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Block not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_block_receipts(
    State(state): State<ServerState>,
    Path(height): Path<u64>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<ReceiptsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    let block_height = height;
    query.set_block_height(block_height);
    let response: GetDataResponse = Receipt::find_many(&state.db.pool, &query)
        .await?
        .try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{height}/inputs",
    tag = TAG_BLOCKS,
    params(
        ("height" = BlockHeight, Path, description = "Block height"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("input_index" = Option<i32>, Query, description = "Filter by input index"),
        ("input_type" = Option<InputType>, Query, description = "Filter by input type"),
        ("owner_id" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
        ("asset_id" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
        ("contract_id" = Option<ContractId>, Query, description = "Filter by contract ID (for contract inputs)"),
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
        (status = 200, description = "Successfully retrieved block inputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Block not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_block_inputs(
    State(state): State<ServerState>,
    Path(height): Path<u64>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<InputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    let block_height = height;
    query.set_block_height(block_height);
    let response: GetDataResponse =
        Input::find_many(&state.db.pool, &query).await?.try_into()?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{height}/outputs",
    tag = TAG_BLOCKS,
    params(
        ("height" = BlockHeight, Path, description = "Block height"),
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("output_index" = Option<i32>, Query, description = "Filter by output index"),
        ("output_type" = Option<OutputType>, Query, description = "Filter by output type"),
        ("to_address" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
        ("asset_id" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
        ("contract_id" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
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
        (status = 200, description = "Successfully retrieved block outputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Block not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_block_outputs(
    State(state): State<ServerState>,
    Path(height): Path<u64>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<OutputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    let block_height = height;
    query.set_block_height(block_height);
    let response: GetDataResponse = Output::find_many(&state.db.pool, &query)
        .await?
        .try_into()?;
    Ok(Json(response))
}

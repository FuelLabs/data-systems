use axum::{
    extract::{FromRequest, Path, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    blocks::{BlocksQuery, TimeRange},
    infra::repository::{Repository, ValidatedQuery},
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
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by timestamp"),
        ("timeRange" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("after" = Option<i32>, Query, description = "Return blocks after this height"),
        ("before" = Option<i32>, Query, description = "Return blocks before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100),
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
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("txStatus" = Option<TransactionStatus>, Query, description = "Filter by transaction status"),
        ("type" = Option<TransactionType>, Query, description = "Filter by transaction type"),
        ("after" = Option<i32>, Query, description = "Return transactions after this height"),
        ("before" = Option<i32>, Query, description = "Return transactions before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
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
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("receiptIndex" = Option<i32>, Query, description = "Filter by receipt index"),
        ("receiptType" = Option<ReceiptType>, Query, description = "Filter by receipt type"),
        ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
        ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
        ("contract" = Option<ContractId>, Query, description = "Filter by contract ID"),
        ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
        ("sender" = Option<Address>, Query, description = "Filter by sender address"),
        ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
        ("subId" = Option<Bytes32>, Query, description = "Filter by sub ID"),
        ("address" = Option<Address>, Query, description = "Filter by address (for accounts)"),
        ("after" = Option<i32>, Query, description = "Return receipts after this height"),
        ("before" = Option<i32>, Query, description = "Return receipts before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
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
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("inputType" = Option<InputType>, Query, description = "Filter by input type"),
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
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("outputIndex" = Option<i32>, Query, description = "Filter by output index"),
        ("outputType" = Option<OutputType>, Query, description = "Filter by output type"),
        ("toAddress" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
        ("address" = Option<Address>, Query, description = "Filter by address (for accounts)"),
        ("after" = Option<i32>, Query, description = "Return outputs after this height"),
        ("before" = Option<i32>, Query, description = "Return outputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
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

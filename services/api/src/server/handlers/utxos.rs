use axum::{
    extract::{FromRequest, State},
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
    utxos::UtxosQuery,
};

use super::open_api::TAG_UTXOS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/utxos",
    tag = TAG_UTXOS,
    params(
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
        ("contract_id" = Option<ContractId>, Query, description = "Filter by contract ID"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return UTXOs after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return UTXOs before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 50),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 50),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 50),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved UTXOs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "No UTXOs found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_utxos(
    State(state): State<ServerState>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let query = ValidatedQuery::<UtxosQuery>::from_request(req, &state)
        .await?
        .into_inner();
    let response: GetDataResponse =
        Utxo::find_many(&state.db.pool, &query).await?.try_into()?;
    Ok(Json(response))
}

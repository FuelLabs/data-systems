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
    predicates::PredicatesQuery,
};

use super::open_api::TAG_PREDICATES;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/predicates",
    tag = TAG_PREDICATES,
    params(
        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
        ("input_index" = Option<i32>, Query, description = "Filter by input index"),
        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("blob_id" = Option<HexData>, Query, description = "Filter by blob ID"),
        ("predicate_address" = Option<Address>, Query, description = "Filter by predicate address"),
        ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return predicates after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return predicates before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 50),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 50),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 50),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved predicates", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "No predicates found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_predicates(
    State(state): State<ServerState>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let query = ValidatedQuery::<PredicatesQuery>::from_request(req, &state)
        .await?
        .into_inner();
    let response: GetDataResponse =
        Predicate::find_many(&state.db.pool, &query)
            .await?
            .try_into()?;
    Ok(Json(response))
}

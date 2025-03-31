use axum::{
    extract::{FromRequest, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    infra::repository::{Repository, ValidatedQuery},
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
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("blobId" = Option<String>, Query, description = "Filter by blob ID"),
        ("predicateAddress" = Option<Address>, Query, description = "Filter by predicate address"),
        ("after" = Option<i32>, Query, description = "Return predicates after this height"),
        ("before" = Option<i32>, Query, description = "Return predicates before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved predicates", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
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

use axum::{
    extract::{FromRequest, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::{
    Address,
    BlockHeight,
    ContractId,
    HexData,
    InputType,
    TxId,
};
use fuel_streams_domains::{
    queryable::{Queryable, ValidatedQuery},
    utxos::queryable::UtxosQuery,
};

use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/utxos",
    tag = "utxos",
    params(
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("utxoType" = Option<InputType>, Query, description = "Filter by UTXO type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("utxoId" = Option<HexData>, Query, description = "Filter by UTXO ID"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID"),
        ("address" = Option<Address>, Query, description = "Filter by address"),
        ("after" = Option<i32>, Query, description = "Return UTXOs after this height"),
        ("before" = Option<i32>, Query, description = "Return UTXOs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved UTXOs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
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
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}

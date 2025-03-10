use actix_web::{web, HttpRequest, HttpResponse};
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
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

#[utoipa::path(
    get,
    path = "/utxos",
    tag = "utxos",
    params(
        // UtxosQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("utxoType" = Option<InputType>, Query, description = "Filter by UTXO type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("utxoId" = Option<HexData>, Query, description = "Filter by UTXO ID"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID"),
        ("address" = Option<Address>, Query, description = "Filter by address"),
        // Flattened QueryPagination fields
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
    req: HttpRequest,
    req_query: ValidatedQuery<UtxosQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let query = req_query.into_inner();
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

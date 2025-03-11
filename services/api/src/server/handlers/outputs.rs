use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_core::types::{
    Address,
    AssetId,
    BlockHeight,
    ContractId,
    TxId,
};
use fuel_streams_domains::{
    outputs::{queryable::OutputsQuery, OutputType},
    queryable::{Queryable, ValidatedQuery},
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

#[utoipa::path(
    get,
    path = "/outputs",
    tag = "outputs",
    params(
        // OutputsQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("outputIndex" = Option<i32>, Query, description = "Filter by output index"),
        ("outputType" = Option<OutputType>, Query, description = "Filter by output type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("toAddress" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
        ("address" = Option<Address>, Query, description = "Filter by address"),
        // Flattened QueryPagination fields
        ("after" = Option<i32>, Query, description = "Return outputs after this height"),
        ("before" = Option<i32>, Query, description = "Return outputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved outputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_outputs(
    req: HttpRequest,
    req_query: ValidatedQuery<OutputsQuery>,
    state: web::Data<ServerState>,
    queried_output_type: Option<OutputType>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    query.set_output_type(queried_output_type);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

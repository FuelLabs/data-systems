use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_core::types::{
    Address,
    AssetId,
    BlockHeight,
    ContractId,
    TxId,
};
use fuel_streams_domains::{
    inputs::{queryable::InputsQuery, InputType},
    queryable::{Queryable, ValidatedQuery},
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

#[utoipa::path(
    get,
    path = "/inputs",
    tag = "inputs",
    params(
        // InputsQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("inputType" = Option<InputType>, Query, description = "Filter by input type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("ownerId" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract inputs)"),
        ("senderAddress" = Option<Address>, Query, description = "Filter by sender address (for message inputs)"),
        ("recipientAddress" = Option<Address>, Query, description = "Filter by recipient address (for message inputs)"),
        ("address" = Option<Address>, Query, description = "Filter by address"),
        // Flattened QueryPagination fields
        ("after" = Option<i32>, Query, description = "Return inputs after this height"),
        ("before" = Option<i32>, Query, description = "Return inputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved inputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_inputs(
    req: HttpRequest,
    req_query: ValidatedQuery<InputsQuery>,
    state: web::Data<ServerState>,
    queried_input_type: Option<InputType>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    query.set_input_type(queried_input_type);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

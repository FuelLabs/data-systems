use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_core::types::{
    Address,
    AssetId,
    BlockHeight,
    Bytes32,
    ContractId,
    TxId,
};
use fuel_streams_domains::{
    queryable::{Queryable, ValidatedQuery},
    receipts::{queryable::ReceiptsQuery, ReceiptType},
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

#[utoipa::path(
    get,
    path = "/receipts",
    tag = "receipts",
    params(
        // ReceiptsQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("receiptIndex" = Option<i32>, Query, description = "Filter by receipt index"),
        ("receiptType" = Option<ReceiptType>, Query, description = "Filter by receipt type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
        ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
        ("contract" = Option<ContractId>, Query, description = "Filter by contract ID"),
        ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
        ("sender" = Option<Address>, Query, description = "Filter by sender address"),
        ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
        ("subId" = Option<Bytes32>, Query, description = "Filter by sub ID"),
        ("address" = Option<Address>, Query, description = "Filter by address"),
        // Flattened QueryPagination fields
        ("after" = Option<i32>, Query, description = "Return receipts after this height"),
        ("before" = Option<i32>, Query, description = "Return receipts before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved receipts", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_receipts(
    req: HttpRequest,
    req_query: ValidatedQuery<ReceiptsQuery>,
    state: web::Data<ServerState>,
    queried_receipt_type: Option<ReceiptType>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    query.set_receipt_type(queried_receipt_type);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

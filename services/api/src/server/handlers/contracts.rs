use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_core::types::{
    Address,
    AssetId,
    BlockHeight,
    HexData,
    InputType,
    OutputType,
    TransactionStatus,
    TransactionType,
    TxId,
};
use fuel_streams_domains::{
    inputs::queryable::InputsQuery,
    outputs::queryable::OutputsQuery,
    queryable::{Queryable, ValidatedQuery},
    transactions::queryable::TransactionsQuery,
    utxos::queryable::UtxosQuery,
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/transactions",
    tag = "contracts",
    params(
        // Path parameter
        ("contractId" = String, Path, description = "Contract ID"),
        // TransactionsQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("txStatus" = Option<TransactionStatus>, Query, description = "Filter by transaction status"),
        ("type" = Option<TransactionType>, Query, description = "Filter by transaction type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        // Flattened QueryPagination fields
        ("after" = Option<i32>, Query, description = "Return transactions after this height"),
        ("before" = Option<i32>, Query, description = "Return transactions before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract transactions", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_transactions(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: ValidatedQuery<TransactionsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/inputs",
    tag = "contracts",
    params(
        // Path parameter
        ("contractId" = String, Path, description = "Contract ID"),
        // InputsQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("inputType" = Option<InputType>, Query, description = "Filter by input type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("ownerId" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
        ("senderAddress" = Option<Address>, Query, description = "Filter by sender address (for message inputs)"),
        ("recipientAddress" = Option<Address>, Query, description = "Filter by recipient address (for message inputs)"),
        // Flattened QueryPagination fields
        ("after" = Option<i32>, Query, description = "Return inputs after this height"),
        ("before" = Option<i32>, Query, description = "Return inputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract inputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_inputs(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: ValidatedQuery<InputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/outputs",
    tag = "contracts",
    params(
        // Path parameter
        ("contractId" = String, Path, description = "Contract ID"),
        // OutputsQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("outputIndex" = Option<i32>, Query, description = "Filter by output index"),
        ("outputType" = Option<OutputType>, Query, description = "Filter by output type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("toAddress" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
        // Flattened QueryPagination fields
        ("after" = Option<i32>, Query, description = "Return outputs after this height"),
        ("before" = Option<i32>, Query, description = "Return outputs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract outputs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_outputs(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: ValidatedQuery<OutputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/contracts/{contractId}/utxos",
    tag = "contracts",
    params(
        // Path parameter
        ("contractId" = String, Path, description = "Contract ID"),
        // UtxosQuery fields
        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
        ("utxoType" = Option<InputType>, Query, description = "Filter by UTXO type"),
        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("utxoId" = Option<HexData>, Query, description = "Filter by UTXO ID"),
        // Flattened QueryPagination fields
        ("after" = Option<i32>, Query, description = "Return UTXOs after this height"),
        ("before" = Option<i32>, Query, description = "Return UTXOs before this height"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
    ),
    responses(
        (status = 200, description = "Successfully retrieved contract UTXOs", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "Contract not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_contracts_utxos(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: ValidatedQuery<UtxosQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

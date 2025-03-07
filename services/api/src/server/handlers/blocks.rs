use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    blocks::queryable::BlocksQuery,
    inputs::queryable::InputsQuery,
    outputs::queryable::OutputsQuery,
    queryable::{Queryable, ValidatedQuery},
    receipts::queryable::ReceiptsQuery,
    transactions::queryable::TransactionsQuery,
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

pub async fn get_blocks(
    req: HttpRequest,
    req_query: ValidatedQuery<BlocksQuery>,
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

pub async fn get_block_transactions(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: ValidatedQuery<TransactionsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_block_receipts(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: ValidatedQuery<ReceiptsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_block_inputs(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: ValidatedQuery<InputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_block_outputs(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: ValidatedQuery<OutputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    blocks::{queryable::BlocksQuery, BlockDbItem},
    inputs::{queryable::InputsQuery, InputDbItem},
    outputs::{queryable::OutputsQuery, OutputDbItem},
    queryable::Queryable,
    receipts::{queryable::ReceiptsQuery, ReceiptDbItem},
    transactions::{queryable::TransactionsQuery, TransactionDbItem},
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDbEntityResponse};
use crate::server::state::ServerState;

pub async fn get_blocks(
    req: HttpRequest,
    req_query: web::Query<BlocksQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let query = req_query.into_inner();
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<BlockDbItem> { data: db_records }))
}

pub async fn get_block_transactions(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: web::Query<TransactionsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<TransactionDbItem> { data: db_records }))
}

pub async fn get_block_receipts(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: web::Query<ReceiptsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<ReceiptDbItem> { data: db_records }))
}

pub async fn get_block_inputs(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: web::Query<InputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<InputDbItem> { data: db_records }))
}

pub async fn get_block_outputs(
    req: HttpRequest,
    height: web::Path<u64>,
    req_query: web::Query<OutputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let block_height = height.into_inner();
    query.set_block_height(block_height);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<OutputDbItem> { data: db_records }))
}

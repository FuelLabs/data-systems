use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    inputs::{queryable::InputsQuery, InputDbItem},
    outputs::{queryable::OutputsQuery, OutputDbItem},
    queryable::Queryable,
    receipts::{queryable::ReceiptsQuery, ReceiptDbItem},
    transactions::{queryable::TransactionsQuery, TransactionDbItem},
};
use fuel_web_utils::server::middlewares::api_key::ApiKey;

use super::{Error, GetDbEntityResponse};
use crate::server::state::ServerState;

pub async fn get_transactions(
    req: HttpRequest,
    req_query: web::Query<TransactionsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let query = req_query.into_inner();
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<TransactionDbItem> { data: db_records }))
}

pub async fn get_transaction_receipts(
    req: HttpRequest,
    tx_id: web::Path<String>,
    req_query: web::Query<ReceiptsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let tx_id = tx_id.into_inner();
    query.set_tx_id(&tx_id);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<ReceiptDbItem> { data: db_records }))
}

pub async fn get_transaction_inputs(
    req: HttpRequest,
    tx_id: web::Path<String>,
    req_query: web::Query<InputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let tx_id = tx_id.into_inner();
    query.set_tx_id(&tx_id);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<InputDbItem> { data: db_records }))
}

pub async fn get_transaction_outputs(
    req: HttpRequest,
    tx_id: web::Path<String>,
    req_query: web::Query<OutputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let tx_id = tx_id.into_inner();
    query.set_tx_id(&tx_id);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<OutputDbItem> { data: db_records }))
}

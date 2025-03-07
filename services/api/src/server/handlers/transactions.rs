use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    inputs::queryable::InputsQuery,
    outputs::queryable::OutputsQuery,
    queryable::{Queryable, ValidatedQuery},
    receipts::queryable::ReceiptsQuery,
    transactions::queryable::TransactionsQuery,
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

pub async fn get_transactions(
    req: HttpRequest,
    req_query: ValidatedQuery<TransactionsQuery>,
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

pub async fn get_transaction_receipts(
    req: HttpRequest,
    tx_id: web::Path<String>,
    req_query: ValidatedQuery<ReceiptsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let tx_id = tx_id.into_inner();
    query.set_tx_id(&tx_id);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_transaction_inputs(
    req: HttpRequest,
    tx_id: web::Path<String>,
    req_query: ValidatedQuery<InputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let tx_id = tx_id.into_inner();
    query.set_tx_id(&tx_id);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_transaction_outputs(
    req: HttpRequest,
    tx_id: web::Path<String>,
    req_query: ValidatedQuery<OutputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let tx_id = tx_id.into_inner();
    query.set_tx_id(&tx_id);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

use actix_web::{web, HttpRequest, HttpResponse};
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

pub async fn get_accounts_transactions(
    req: HttpRequest,
    address: web::Path<String>,
    req_query: ValidatedQuery<TransactionsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let address = address.into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_accounts_inputs(
    req: HttpRequest,
    address: web::Path<String>,
    req_query: ValidatedQuery<InputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let address = address.into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_accounts_outputs(
    req: HttpRequest,
    address: web::Path<String>,
    req_query: ValidatedQuery<OutputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let address = address.into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_accounts_utxos(
    req: HttpRequest,
    address: web::Path<String>,
    req_query: ValidatedQuery<UtxosQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let address = address.into_inner();
    query.set_address(&address);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

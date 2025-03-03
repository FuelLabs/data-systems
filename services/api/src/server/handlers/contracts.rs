use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    inputs::{queryable::InputsQuery, InputDbItem},
    outputs::{queryable::OutputsQuery, OutputDbItem},
    queryable::Queryable,
    transactions::{queryable::TransactionsQuery, TransactionDbItem},
    utxos::{queryable::UtxosQuery, UtxoDbItem},
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDbEntityResponse};
use crate::server::state::ServerState;

pub async fn get_contracts_transactions(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: web::Query<TransactionsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<TransactionDbItem> { data: db_records }))
}

pub async fn get_contracts_inputs(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: web::Query<InputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<InputDbItem> { data: db_records }))
}

pub async fn get_contracts_outputs(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: web::Query<OutputsQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<OutputDbItem> { data: db_records }))
}

pub async fn get_contracts_utxos(
    req: HttpRequest,
    contract_id: web::Path<String>,
    req_query: web::Query<UtxosQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    let contract_id = contract_id.into_inner();
    query.set_contract_id(&contract_id);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<UtxoDbItem> { data: db_records }))
}

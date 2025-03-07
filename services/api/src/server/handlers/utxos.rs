use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    queryable::{Queryable, ValidatedQuery},
    utxos::queryable::UtxosQuery,
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

pub async fn get_utxos(
    req: HttpRequest,
    req_query: ValidatedQuery<UtxosQuery>,
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

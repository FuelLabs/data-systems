use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    queryable::Queryable,
    utxos::{queryable::UtxosQuery, UtxoDbItem},
};
use fuel_web_utils::server::middlewares::api_key::ApiKey;

use super::{Error, GetDbEntityResponse};
use crate::server::state::ServerState;

pub async fn get_utxos(
    req: HttpRequest,
    req_query: web::Query<UtxosQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let query = req_query.into_inner();
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<UtxoDbItem> { data: db_records }))
}

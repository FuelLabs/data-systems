use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    outputs::{queryable::OutputsQuery, OutputDbItem, OutputType},
    queryable::Queryable,
};
use fuel_web_utils::server::middlewares::api_key::ApiKey;

use super::{Error, GetDbEntityResponse};
use crate::server::state::ServerState;

pub async fn get_outputs(
    req: HttpRequest,
    req_query: web::Query<OutputsQuery>,
    state: web::Data<ServerState>,
    queried_output_type: Option<OutputType>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    query.set_output_type(queried_output_type);
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok()
        .json(GetDbEntityResponse::<OutputDbItem> { data: db_records }))
}

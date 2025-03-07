use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    inputs::{queryable::InputsQuery, InputType},
    queryable::{Queryable, ValidatedQuery},
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

pub async fn get_inputs(
    req: HttpRequest,
    req_query: ValidatedQuery<InputsQuery>,
    state: web::Data<ServerState>,
    queried_input_type: Option<InputType>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    query.set_input_type(queried_input_type);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}

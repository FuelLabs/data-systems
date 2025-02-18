use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    blocks::{queryable::BlocksQuery, BlockDbItem},
    queryable::Queryable,
};
use fuel_web_utils::server::middlewares::api_key::ApiKey;
use serde::{Deserialize, Serialize};

use crate::server::state::ServerState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Validation error {0}")]
    Validation(#[from] validator::ValidationErrors),
}

impl From<Error> for actix_web::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Sqlx(e) => actix_web::error::InternalError::new(
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into(),
            Error::Validation(e) => {
                actix_web::error::InternalError::new(e, StatusCode::BAD_REQUEST)
                    .into()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlocksTestResponse {
    data: Vec<BlockDbItem>,
}

pub async fn get_blocks(
    req: HttpRequest,
    req_query: web::Query<BlocksQuery>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let query = req_query.into_inner();
    let db_records =
        query.execute(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok().json(GetBlocksTestResponse { data: db_records }))
}

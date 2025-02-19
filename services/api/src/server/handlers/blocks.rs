use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse, Result};
use fuel_web_utils::server::middlewares::api_key::ApiKey;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

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
pub struct GetBlocksTestRequest {
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlocksTestResponse {
    value: i32,
}

async fn select_one(tx: &Pool<Postgres>) -> Result<i32, sqlx::Error> {
    let value = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(tx)
        .await?;
    Ok(value)
}

pub async fn get_blocks(
    req: HttpRequest,
    req_body: web::Json<GetBlocksTestRequest>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let _req = req_body.into_inner();
    let one = select_one(&state.db.pool).await.map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok().json(GetBlocksTestResponse { value: one }))
}

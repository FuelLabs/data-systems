use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse, Result};
use fuel_web_utils::api_key::{
    ApiKey,
    ApiKeyError,
    ApiKeyRoleName,
    ApiKeyUserName,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use validator::Validate;

use crate::server::state::ServerState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error {0}")]
    ApiKey(#[from] ApiKeyError),
    #[error("Validation error {0}")]
    Validation(#[from] validator::ValidationErrors),
}

impl From<Error> for actix_web::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::ApiKey(e) => actix_web::error::InternalError::new(
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

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GenerateApiKeyRequest {
    pub username: ApiKeyUserName,
    pub role: ApiKeyRoleName,
}

async fn insert_api_key(
    request: &GenerateApiKeyRequest,
    tx: &Pool<Postgres>,
) -> Result<ApiKey, ApiKeyError> {
    let api_key = ApiKey::create(tx, &request.username, &request.role).await?;
    Ok(api_key)
}

pub async fn generate_api_key(
    _req: HttpRequest,
    req_body: web::Json<GenerateApiKeyRequest>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let req = req_body.into_inner();
    req.validate().map_err(Error::Validation)?;
    let db_record = insert_api_key(&req, &state.db.pool).await?;
    Ok(HttpResponse::Ok().json(db_record))
}

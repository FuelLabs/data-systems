use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse, Result};
use fuel_db_utils::generate_random_api_key;
use fuel_web_utils::server::middlewares::api_key::DbUserApiKey;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use validator::Validate;

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

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GenerateApiKeyRequest {
    #[validate(range(min = 1))]
    pub user_id: u32,
    #[validate(length(min = 6))]
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateApiKeyResponse {
    pub user_id: i64,
    pub username: String,
    pub api_key: String,
}

impl From<&DbUserApiKey> for GenerateApiKeyResponse {
    fn from(api_key: &DbUserApiKey) -> Self {
        Self {
            user_id: api_key.user_id,
            username: api_key.user_name.clone(),
            api_key: api_key.api_key.clone(),
        }
    }
}

async fn insert_api_key(
    request: &GenerateApiKeyRequest,
    tx: &Pool<Postgres>,
) -> Result<DbUserApiKey, sqlx::Error> {
    let db_record = sqlx::query_as::<_, DbUserApiKey>(
        "INSERT INTO api_keys (user_id, user_name, api_key)
        VALUES ($1, $2, $3)
        RETURNING user_id, user_name, api_key",
    )
    .bind(request.user_id as i32)
    .bind(&request.username)
    .bind(generate_random_api_key())
    .fetch_one(tx)
    .await?;
    Ok(db_record)
}

pub async fn generate_api_key(
    _req: HttpRequest,
    req_body: web::Json<GenerateApiKeyRequest>,
    state: web::Data<ServerState>,
) -> actix_web::Result<HttpResponse> {
    let req = req_body.into_inner();
    req.validate().map_err(Error::Validation)?;
    // TODO: we need to ensure here that user_id does not already exist
    let db_record = insert_api_key(&req, &state.db.pool)
        .await
        .map_err(Error::Sqlx)?;
    Ok(HttpResponse::Ok().json(GenerateApiKeyResponse::from(&db_record)))
}

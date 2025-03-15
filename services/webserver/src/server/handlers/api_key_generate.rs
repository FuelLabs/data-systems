use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fuel_web_utils::api_key::{
    ApiKey,
    ApiKeyError,
    ApiKeyRoleName,
    ApiKeyUserName,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use validator::Validate;

// Assuming this is your server state struct
use crate::server::state::ServerState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error {0}")]
    ApiKey(#[from] ApiKeyError),
    #[error("Validation error {0}")]
    Validation(#[from] validator::ValidationErrors),
}

// Implement IntoResponse for custom error handling
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::ApiKey(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    .into_response()
            }
            Error::Validation(e) => {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
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
    pool: &Pool<Postgres>,
) -> Result<ApiKey, ApiKeyError> {
    let api_key =
        ApiKey::create(pool, &request.username, &request.role).await?;
    Ok(api_key)
}

// Handler function for Axum
pub async fn generate_api_key(
    State(state): State<ServerState>,
    Json(req): Json<GenerateApiKeyRequest>,
) -> Result<Json<ApiKey>, Error> {
    req.validate().map_err(Error::Validation)?;
    let db_record = insert_api_key(&req, &state.db.pool).await?;
    Ok(Json(db_record))
}

use axum::{
    body::Body,
    extract::{Json, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use fuel_web_utils::api_key::{
    ApiKey,
    ApiKeyError,
    ApiKeyRoleName,
    ApiKeyUserName,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::open_api::TAG_API_KEYS;
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

#[derive(Debug, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct GenerateApiKeyRequest {
    pub username: ApiKeyUserName,
    pub role: ApiKeyRoleName,
}

#[utoipa::path(
    post,
    path = "/keys/generate",
    tag = TAG_API_KEYS,
    responses(
        (status = 200, description = "Successfully generated api key", body = GenerateApiKeyRequest),
        (status = 400, description = "Invalid body", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn generate_api_key(
    State(state): State<ServerState>,
    Json(req): Json<GenerateApiKeyRequest>,
) -> Result<Json<ApiKey>, Error> {
    req.validate().map_err(Error::Validation)?;
    let api_key =
        ApiKey::create(state.db.pool_ref(), &req.username, &req.role).await?;
    Ok(Json(api_key))
}

pub async fn validate_manage_api_keys_scope(
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiKeyError> {
    let api_key = ApiKey::from_req(&req)?;
    if !api_key
        .scopes()
        .iter()
        .any(|scope| scope.is_manage_api_keys())
    {
        tracing::warn!(
            id = %api_key.id(),
            user = %api_key.user(),
            "API key missing MANAGE_API_KEYS scope"
        );
        return Err(ApiKeyError::ScopePermission("MANAGE_API_KEYS".to_string()));
    }

    Ok(next.run(req).await)
}

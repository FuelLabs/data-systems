use axum::{
    extract::rejection::QueryRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::ApiKeyStorageError;

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("API key not found in request")]
    NotFound,
    #[error("API key is invalid or expired")]
    Invalid,
    #[error("API key status is invalid: {0}")]
    BadStatus(String),
    #[error("Failed to decode SQLx value: {0}")]
    SqlxDecode(#[source] sqlx::error::BoxDynError),
    #[error(transparent)]
    Storage(#[from] ApiKeyStorageError),
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
    #[error("API key status is invalid: {0}")]
    InvalidStatus(String),
    #[error("API key role permission is invalid: {0}")]
    RolePermission(String),
    #[error("API key scope permission is invalid: {0}")]
    ScopePermission(String),
    #[error("API key format is invalid: {0}")]
    InvalidKeyFormat(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Subscription limit exceeded: {0}")]
    SubscriptionLimitExceeded(String),
    #[error("API key rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("Historical limit exceeded: {0}")]
    HistoricalLimitExceeded(String),
    #[error(transparent)]
    Query(#[from] QueryRejection),
}

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        match self {
            // Unauthorized errors (401)
            ApiKeyError::NotFound => {
                (StatusCode::UNAUTHORIZED, "").into_response()
            }
            ApiKeyError::Invalid
            | ApiKeyError::InvalidHeader(_)
            | ApiKeyError::RolePermission(_)
            | ApiKeyError::ScopePermission(_)
            | ApiKeyError::InvalidKeyFormat(_)
            | ApiKeyError::InvalidStatus(_)
            | ApiKeyError::HistoricalLimitExceeded(_)
            | ApiKeyError::Query(_) => {
                (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            }

            // Forbidden errors (403)
            ApiKeyError::BadStatus(_) => {
                (StatusCode::FORBIDDEN, self.to_string()).into_response()
            }

            // Internal server errors (500)
            ApiKeyError::Storage(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    .into_response()
            }
            ApiKeyError::SqlxDecode(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    .into_response()
            }
            ApiKeyError::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
            }

            // Too many requests (429)
            ApiKeyError::RateLimitExceeded(info)
            | ApiKeyError::SubscriptionLimitExceeded(info) => {
                (StatusCode::TOO_MANY_REQUESTS, info).into_response()
            }
        }
    }
}

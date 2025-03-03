use actix_web::http::header::InvalidHeaderValue;

use super::{ApiKeyManagerError, ApiKeyStorageError};

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
    #[error(transparent)]
    Manager(#[from] ApiKeyManagerError),
    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderValue),
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
}

impl From<ApiKeyError> for actix_web::Error {
    fn from(err: ApiKeyError) -> Self {
        match err {
            // Unauthorized errors
            ApiKeyError::NotFound => actix_web::error::ErrorUnauthorized(""),
            ApiKeyError::Invalid
            | ApiKeyError::InvalidHeader(_)
            | ApiKeyError::RolePermission(_)
            | ApiKeyError::ScopePermission(_)
            | ApiKeyError::InvalidKeyFormat(_)
            | ApiKeyError::InvalidStatus(_)
            | ApiKeyError::HistoricalLimitExceeded(_) => {
                actix_web::error::ErrorUnauthorized(err.to_string())
            }

            // Forbidden errors
            ApiKeyError::BadStatus(_) => {
                actix_web::error::ErrorForbidden(err.to_string())
            }

            // Internal server errors
            ApiKeyError::Storage(e) => {
                actix_web::error::ErrorInternalServerError(e.to_string())
            }
            ApiKeyError::Manager(e) => {
                actix_web::error::ErrorInternalServerError(e.to_string())
            }
            ApiKeyError::SqlxDecode(e) => {
                actix_web::error::ErrorInternalServerError(e.to_string())
            }
            ApiKeyError::DatabaseError(e) => {
                actix_web::error::ErrorInternalServerError(e)
            }

            // Rate limit error
            ApiKeyError::RateLimitExceeded(info)
            | ApiKeyError::SubscriptionLimitExceeded(info) => {
                actix_web::error::ErrorTooManyRequests(info)
            }
        }
    }
}

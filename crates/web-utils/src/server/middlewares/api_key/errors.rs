use actix_web::http::header::InvalidHeaderValue;

use super::{ApiKeyManagerError, ApiKeyStorageError};

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("API key not found in request")]
    NotFound,
    #[error("API key is invalid or expired")]
    Invalid,
    #[error("API key is inactive")]
    Inactive,
    #[error("API key is deleted")]
    Deleted,
    #[error(transparent)]
    Storage(#[from] ApiKeyStorageError),
    #[error(transparent)]
    Manager(#[from] ApiKeyManagerError),
    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderValue),
}

impl From<ApiKeyError> for actix_web::Error {
    fn from(err: ApiKeyError) -> Self {
        match err {
            ApiKeyError::NotFound => {
                actix_web::error::ErrorUnauthorized("API key not found")
            }
            ApiKeyError::Invalid => {
                actix_web::error::ErrorUnauthorized("Invalid API key")
            }
            ApiKeyError::Inactive => {
                actix_web::error::ErrorForbidden("API key is inactive")
            }
            ApiKeyError::Deleted => {
                actix_web::error::ErrorForbidden("API key is deleted")
            }
            ApiKeyError::Storage(e) => {
                actix_web::error::ErrorInternalServerError(e.to_string())
            }
            ApiKeyError::Manager(e) => {
                actix_web::error::ErrorInternalServerError(e.to_string())
            }
            ApiKeyError::InvalidHeader(e) => {
                actix_web::error::ErrorUnauthorized(e.to_string())
            }
        }
    }
}

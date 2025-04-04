use axum::response::{IntoResponse, Response};
use fuel_streams_core::types::StreamResponseError;
use fuel_streams_domains::infra::{
    repository::{RepositoryError, ValidatedQueryError},
    DbError,
};
use fuel_web_utils::api_key::ApiKeyError;
use tokio::task::JoinError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    JoinHandle(#[from] JoinError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),
    #[error(transparent)]
    Stream(#[from] StreamResponseError),
    #[error(transparent)]
    ApiKey(#[from] ApiKeyError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    ValidatedQuery(#[from] ValidatedQueryError),
    #[error("Invalid contract id: {0}")]
    InvalidContractId(String),
}

// Implement IntoResponse for custom error handling
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            // Client errors - Bad Request
            ApiError::Validation(e) => {
                (axum::http::StatusCode::BAD_REQUEST, e.to_string())
                    .into_response()
            }
            ApiError::ValidatedQuery(e) => {
                (axum::http::StatusCode::BAD_REQUEST, e.to_string())
                    .into_response()
            }
            ApiError::InvalidContractId(e) => {
                (axum::http::StatusCode::BAD_REQUEST, e).into_response()
            }

            // Database related errors
            ApiError::Database(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response(),
            ApiError::Sqlx(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response(),

            // API and authentication errors
            ApiError::ApiKey(e) => (
                axum::http::StatusCode::UNAUTHORIZED,
                format!("API key error: {}", e),
            )
                .into_response(),

            // Processing errors
            ApiError::Stream(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Stream error: {}", e),
            )
                .into_response(),
            ApiError::JoinHandle(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Task error: {}", e),
            )
                .into_response(),
            ApiError::Serde(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Serialization error: {}", e),
            )
                .into_response(),

            // Handle the new HttpError variant
            ApiError::Repository(e) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Repository error: {}", e),
            )
                .into_response(),
        }
    }
}

use fuel_streams_store::db::DbError;
use tokio::task::JoinError;

/// api-related errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    JoinHandle(#[from] JoinError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Database(#[from] DbError),
}

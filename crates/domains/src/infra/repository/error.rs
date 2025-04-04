use fuel_data_parser::DataParserError;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Failed to insert item: {0}")]
    Insert(#[source] sqlx::Error),
    #[error(transparent)]
    Encode(#[from] DataParserError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

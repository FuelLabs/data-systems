#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to open database")]
    Open(#[source] sqlx::Error),
    #[error("Failed to insert data")]
    Insert(#[source] sqlx::Error),
    #[error("Duplicate subject: {0}")]
    DuplicateSubject(String),
    #[error("Failed to update data")]
    Update(#[source] sqlx::Error),
    #[error("Failed to upsert data")]
    Upsert(#[source] sqlx::Error),
    #[error("Failed to delete data")]
    Delete(#[source] sqlx::Error),
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Failed to find record")]
    Find(#[source] sqlx::Error),
    #[error("Failed to query data")]
    Query(#[source] sqlx::Error),
}

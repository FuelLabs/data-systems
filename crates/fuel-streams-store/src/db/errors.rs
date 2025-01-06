#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Cockroach db error: {0}")]
    Cockroach(#[from] CockroachDbError),
}

#[derive(thiserror::Error, Debug)]
pub enum CockroachDbError {
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
    #[error("Failed to query data")]
    Query(#[source] sqlx::Error),
}

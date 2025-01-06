use crate::{db::DbError, subject_validator::SubjectPatternError};

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("Db error: {0}")]
    Db(#[from] DbError),
    #[error("Invalid subject pattern '{pattern}': {error}")]
    InvalidSubject {
        pattern: String,
        error: SubjectPatternError,
    },
}

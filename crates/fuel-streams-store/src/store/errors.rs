use crate::{storage::StorageError, subject_validator::SubjectPatternError};

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Invalid subject pattern '{pattern}': {error}")]
    InvalidSubject {
        pattern: String,
        error: SubjectPatternError,
    },
}

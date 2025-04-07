use fuel_streams_domains::infra::{DbError, RepositoryError};
use thiserror::Error as ThisError;

use crate::{helpers::AvroParserError, s3::StorageError};

#[derive(ThisError, Debug)]
pub enum DuneError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Avro(#[from] AvroParserError),
    #[error(transparent)]
    S3(#[from] StorageError),
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

pub type DuneResult<T> = Result<T, DuneError>;

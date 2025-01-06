use super::StorageError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageRecord {
    pub subject: String,
    pub value: Vec<u8>,
}

pub type StorageResult<T> = Result<T, StorageError>;

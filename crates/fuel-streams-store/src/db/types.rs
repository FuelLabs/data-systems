use super::DbError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbRecord {
    pub subject: String,
    pub value: Vec<u8>,
}

pub type DbResult<T> = Result<T, DbError>;

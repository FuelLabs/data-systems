use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use super::StoreError;
use crate::db::DbRecord;

#[async_trait]
pub trait Recordable:
    Clone + Send + Sync + Sized + Serialize + DeserializeOwned + 'static
{
    fn to_record(&self, subject: &str) -> StoreRecord<Self> {
        StoreRecord::new(subject, self.clone())
    }
    fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct StoreRecord<T: Recordable> {
    pub subject: String,
    pub payload: Arc<T>,
}

impl<T: Recordable> StoreRecord<T> {
    pub fn new(subject: &str, payload: T) -> Self {
        Self {
            subject: subject.to_string(),
            payload: Arc::new(payload),
        }
    }
}

impl<T: Recordable> From<DbRecord> for StoreRecord<T> {
    fn from(record: DbRecord) -> Self {
        Self {
            subject: record.subject,
            payload: Arc::new(Recordable::deserialize(&record.value)),
        }
    }
}

pub type StoreResult<T> = Result<T, StoreError>;

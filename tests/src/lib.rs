use fuel_streams_store::{
    storage::{CockroachConnectionOpts, CockroachStorage, StorageResult},
    store::{Recordable, Store, StoreRecord, StoreResult},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestRecord(pub String);
impl Recordable for TestRecord {}
impl TestRecord {
    pub fn new(payload: impl Into<String>) -> Self {
        Self(payload.into())
    }
}

pub async fn create_test_storage() -> StorageResult<CockroachStorage> {
    let opts = CockroachConnectionOpts::default();
    CockroachStorage::new(opts).await
}

pub async fn setup_store<T: Recordable>() -> StoreResult<Store<T>> {
    let opts = CockroachConnectionOpts::default();
    Store::new(opts).await
}

pub fn create_test_record<T: Recordable>(
    subject: &str,
    payload: T,
) -> StoreRecord<T> {
    StoreRecord::new(subject, payload)
}

pub async fn add_test_records<T: Recordable>(
    store: &Store<T>,
    prefix: &str,
    records: &[(impl AsRef<str>, T)],
) -> StoreResult<()> {
    for (suffix, payload) in records {
        let key = format!("{}.{}", prefix, suffix.as_ref());
        store
            .add_record(&create_test_record(&key, payload.clone()))
            .await?;
    }
    Ok(())
}

pub fn create_random_db_name() -> String {
    format!("test_{}", rand::thread_rng().gen_range(0..1000000))
}

pub async fn cleanup_tables() -> StoreResult<()> {
    let opts = CockroachConnectionOpts::default();
    let storage = CockroachStorage::new(opts).await?;
    storage.cleanup_tables().await?;
    Ok(())
}

use fuel_streams_store::{
    db::{Db, DbConnectionOpts, DbRecordEntity, DbResult, Record},
    store::{Store, StoreResult},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestRecord(pub String);
impl Record for TestRecord {
    const ENTITY: DbRecordEntity = DbRecordEntity::Block;
    fn order(&self) -> i32 {
        0
    }
}
impl TestRecord {
    pub fn new(payload: impl Into<String>) -> Self {
        Self(payload.into())
    }
}

pub async fn create_test_db() -> DbResult<Db> {
    let opts = DbConnectionOpts::default();
    Db::new(opts).await
}

pub async fn setup_store<R: Record>() -> StoreResult<Store<R>> {
    let opts = DbConnectionOpts::default();
    Store::new(opts).await
}

pub async fn add_test_records<R: Record>(
    store: &Store<R>,
    prefix: &str,
    records: &[(impl AsRef<str>, R)],
) -> StoreResult<()> {
    for (suffix, payload) in records {
        let subject = format!("{}.{}", prefix, suffix.as_ref());
        store.add_record(payload, &subject).await?;
    }
    Ok(())
}

pub fn create_random_db_name() -> String {
    format!("test_{}", rand::thread_rng().gen_range(0..1000000))
}

pub async fn cleanup_tables() -> StoreResult<()> {
    let opts = DbConnectionOpts::default();
    let db = Db::new(opts).await?;
    db.cleanup_tables().await?;
    Ok(())
}

use async_trait::async_trait;

use super::{DbRecord, DbResult};

#[async_trait]
pub trait Db: Send + Sync + 'static {
    async fn insert(&self, subject: &str, value: &[u8]) -> DbResult<DbRecord>;
    async fn update(&self, subject: &str, value: &[u8]) -> DbResult<DbRecord>;
    async fn upsert(&self, subject: &str, value: &[u8]) -> DbResult<DbRecord>;
    async fn delete(&self, subject: &str) -> DbResult<()>;
    async fn find_by_pattern(&self, pattern: &str) -> DbResult<Vec<DbRecord>>;
}

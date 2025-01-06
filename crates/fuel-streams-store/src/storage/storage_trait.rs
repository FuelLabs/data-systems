use async_trait::async_trait;

use super::{StorageRecord, StorageResult};

#[async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn insert(
        &self,
        subject: &str,
        value: &[u8],
    ) -> StorageResult<StorageRecord>;
    async fn update(
        &self,
        subject: &str,
        value: &[u8],
    ) -> StorageResult<StorageRecord>;
    async fn upsert(
        &self,
        subject: &str,
        value: &[u8],
    ) -> StorageResult<StorageRecord>;
    async fn delete(&self, subject: &str) -> StorageResult<()>;
    async fn find_by_pattern(
        &self,
        pattern: &str,
    ) -> StorageResult<Vec<StorageRecord>>;
}

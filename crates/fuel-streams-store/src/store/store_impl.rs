use std::sync::Arc;

use futures::{stream, stream::BoxStream, StreamExt};

use super::{CacheConfig, CacheStats, StoreCache, StoreError, StorePacket};
use crate::{
    db::{Db, DbRecord},
    record::Record,
    subject_validator::SubjectValidator,
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct Store<R: Record> {
    pub db: Arc<Db>,
    pub cache: Arc<StoreCache<R>>,
    _marker: std::marker::PhantomData<R>,
}

impl<R: Record> Store<R> {
    pub fn new(db: &Arc<Db>) -> Self {
        Self {
            db: Arc::clone(db),
            cache: Arc::new(StoreCache::new(CacheConfig::default())),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_cache_config(db: &Arc<Db>, cache_config: CacheConfig) -> Self {
        Self {
            db: Arc::clone(db),
            cache: Arc::new(StoreCache::new(cache_config)),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
}

impl<S: Record> Store<S> {
    fn validate_subject(&self, subject_pattern: &str) -> StoreResult<()> {
        if let Err(error) = SubjectValidator::validate(subject_pattern) {
            return Err(StoreError::InvalidSubject {
                pattern: subject_pattern.to_string(),
                error,
            });
        }
        Ok(())
    }

    pub async fn add_record(
        &self,
        packet: &StorePacket<S>,
    ) -> StoreResult<DbRecord> {
        let db_record = packet
            .record
            .insert(&self.db, &packet.subject, packet.order.clone())
            .await?;
        self.cache.insert(&db_record.subject, &packet.record);
        Ok(db_record)
    }

    pub async fn update_record(
        &self,
        packet: &StorePacket<S>,
    ) -> StoreResult<DbRecord> {
        let db_record = packet
            .record
            .update(&self.db, &packet.subject, packet.order.clone())
            .await?;
        self.cache.insert(&db_record.subject, &packet.record);
        Ok(db_record)
    }

    pub async fn upsert_record(
        &self,
        packet: &StorePacket<S>,
    ) -> StoreResult<DbRecord> {
        let db_record = packet
            .record
            .upsert(&self.db, &packet.subject, packet.order.clone())
            .await?;
        self.cache.insert(&db_record.subject, &packet.record);
        Ok(db_record)
    }

    pub async fn delete_record(
        &self,
        packet: &StorePacket<S>,
    ) -> StoreResult<DbRecord> {
        let db_record = packet.record.delete(&self.db, &packet.subject).await?;
        self.cache.delete(&db_record.subject);
        Ok(db_record)
    }

    pub async fn find_many_by_subject(
        &self,
        subject_pattern: &str,
    ) -> StoreResult<Vec<S>> {
        self.validate_subject(subject_pattern)?;
        let items = S::find_many_by_pattern(&self.db, subject_pattern).await?;
        let mut messages = Vec::with_capacity(items.len());
        for item in items {
            let payload = S::from_db_record(&item).await?;
            if item.subject == subject_pattern {
                self.cache.insert(&item.subject, &payload);
            }
            messages.push(payload);
        }

        Ok(messages)
    }

    pub async fn find_many_by_subject_raw(
        &self,
        subject_pattern: &str,
    ) -> StoreResult<Vec<DbRecord>> {
        self.validate_subject(subject_pattern)?;
        S::find_many_by_pattern(&self.db, subject_pattern)
            .await
            .map_err(StoreError::from)
    }

    pub async fn stream_by_subject(
        &self,
        subject_pattern: &str,
    ) -> StoreResult<BoxStream<'static, StoreResult<S>>> {
        self.validate_subject(subject_pattern)?;
        let items = self.find_many_by_subject(subject_pattern).await?;
        let stream = stream::iter(items)
            .map(|item| async move { Ok(item) })
            .buffered(10);

        Ok(Box::pin(stream))
    }

    pub async fn stream_by_subject_raw(
        &self,
        subject_pattern: &str,
    ) -> StoreResult<BoxStream<'static, StoreResult<DbRecord>>> {
        self.validate_subject(subject_pattern)?;
        let items = self.find_many_by_subject_raw(subject_pattern).await?;
        let stream = stream::iter(items)
            .map(|item| async move { Ok(item) })
            .buffered(10);

        Ok(Box::pin(stream))
    }
}

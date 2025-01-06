use std::sync::Arc;

use super::{CacheConfig, CacheStats, StoreCache, StoreError};
use crate::{
    db::{Db, DbConnectionOpts, DbRecord, Record},
    subject_validator::SubjectValidator,
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Clone)]
pub struct Store<R: Record> {
    pub db: Arc<Db>,
    pub cache: Arc<StoreCache<R>>,
    _marker: std::marker::PhantomData<R>,
}

impl<R: Record> Store<R> {
    pub async fn new(opts: DbConnectionOpts) -> StoreResult<Self> {
        let db = Db::new(opts).await?;
        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(StoreCache::new(CacheConfig::default())),
            _marker: std::marker::PhantomData,
        })
    }

    pub async fn with_cache_config(
        opts: DbConnectionOpts,
        cache_config: CacheConfig,
    ) -> StoreResult<Self> {
        let db = Db::new(opts).await?;
        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(StoreCache::new(cache_config)),
            _marker: std::marker::PhantomData,
        })
    }

    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
}

impl<S: Record> Store<S> {
    pub async fn add_record(
        &self,
        record: &S,
        subject: &str,
    ) -> StoreResult<DbRecord> {
        let db_record = record.insert(&self.db, subject).await?;
        self.cache.insert(&db_record.subject, record);
        Ok(db_record)
    }

    pub async fn update_record(
        &self,
        record: &S,
        subject: &str,
    ) -> StoreResult<DbRecord> {
        let db_record = record.update(&self.db, subject).await?;
        self.cache.insert(&db_record.subject, record);
        Ok(db_record)
    }

    pub async fn upsert_record(
        &self,
        record: &S,
        subject: &str,
    ) -> StoreResult<DbRecord> {
        let db_record = record.upsert(&self.db, subject).await?;
        self.cache.insert(&db_record.subject, record);
        Ok(db_record)
    }

    pub async fn delete_record(
        &self,
        record: &S,
        subject: &str,
    ) -> StoreResult<DbRecord> {
        let db_record = record.delete(&self.db, subject).await?;
        self.cache.delete(&db_record.subject);
        Ok(db_record)
    }

    pub async fn find_many_by_subject(
        &self,
        subject_pattern: &str,
    ) -> StoreResult<Vec<S>> {
        if let Err(error) = SubjectValidator::validate(subject_pattern) {
            return Err(StoreError::InvalidSubject {
                pattern: subject_pattern.to_string(),
                error,
            });
        }

        if !subject_pattern.contains(['*', '>']) {
            if let Some(msg) = self.cache.get(subject_pattern) {
                return Ok(vec![msg]);
            }
        }

        let items = S::find_many_by_pattern(&self.db, subject_pattern).await?;
        let mut messages = Vec::with_capacity(items.len());
        for item in items {
            let payload: S = S::from_db_record(&item);
            if item.subject == subject_pattern {
                self.cache.insert(&item.subject, &payload);
            }
            messages.push(payload);
        }

        Ok(messages)
    }
}

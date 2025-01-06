use std::sync::Arc;

use super::{
    CacheConfig,
    CacheStats,
    Recordable,
    StoreCache,
    StoreError,
    StoreRecord,
    StoreResult,
};
use crate::{
    db::{CockroachConnectionOpts, CockroachDb, Db},
    subject_validator::SubjectValidator,
};

#[derive(Clone)]
pub struct Store<S: Recordable> {
    pub db: Arc<dyn Db>,
    cache: Arc<StoreCache<S>>,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Recordable> Store<S> {
    pub async fn new(opts: CockroachConnectionOpts) -> StoreResult<Self> {
        let db = CockroachDb::new(opts).await?;
        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(StoreCache::new(CacheConfig::default())),
            _marker: std::marker::PhantomData,
        })
    }

    pub async fn with_cache_config(
        opts: CockroachConnectionOpts,
        cache_config: CacheConfig,
    ) -> StoreResult<Self> {
        let db = CockroachDb::new(opts).await?;
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

impl<S: Recordable> Store<S> {
    pub async fn add_record(&self, record: &StoreRecord<S>) -> StoreResult<()> {
        let bytes = Recordable::serialize(&*record.payload);
        self.db.insert(&record.subject, &bytes).await?;
        self.cache.insert(&record.subject, &*record.payload);
        Ok(())
    }

    pub async fn update_record(
        &self,
        record: &StoreRecord<S>,
    ) -> StoreResult<()> {
        let bytes = Recordable::serialize(&*record.payload);
        self.db.update(&record.subject, &bytes).await?;
        self.cache.insert(&record.subject, &*record.payload);
        Ok(())
    }

    pub async fn upsert_record(
        &self,
        record: &StoreRecord<S>,
    ) -> StoreResult<()> {
        let bytes = Recordable::serialize(&*record.payload);
        self.db.upsert(&record.subject, &bytes).await?;
        self.cache.insert(&record.subject, &*record.payload);
        Ok(())
    }

    pub async fn delete_record(&self, subject: &str) -> StoreResult<()> {
        self.db.delete(subject).await?;
        self.cache.delete(subject);
        Ok(())
    }

    pub async fn find_by_subject(
        &self,
        subject_pattern: &str,
    ) -> StoreResult<Vec<S>> {
        if let Err(error) = SubjectValidator::validate(subject_pattern) {
            return Err(StoreError::InvalidSubject {
                pattern: subject_pattern.to_string(),
                error,
            });
        }

        // Try cache first for exact matches (no wildcards)
        if !subject_pattern.contains(['*', '>']) {
            if let Some(msg) = self.cache.get(subject_pattern) {
                return Ok(vec![msg]);
            }
        }

        let items = self.db.find_by_pattern(subject_pattern).await?;
        let mut messages = Vec::with_capacity(items.len());
        for item in items {
            let payload: S = Recordable::deserialize(&item.value);
            if item.subject == subject_pattern {
                self.cache.insert(&item.subject, &payload);
            }
            messages.push(payload);
        }

        Ok(messages)
    }
}

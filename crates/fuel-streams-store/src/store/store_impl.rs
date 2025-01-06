use std::sync::Arc;

use super::{CacheConfig, CacheStats, StoreCache, StoreError};
use crate::{
    db::{Db, DbRecord, Record},
    subject_validator::SubjectValidator,
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Clone)]
pub struct StorePacket<R: Record> {
    pub record: R,
    pub subject: String,
    order: Option<i32>,
}
impl<R: Record> StorePacket<R> {
    pub fn new(record: &R, subject: String) -> Self {
        Self {
            record: record.to_owned(),
            subject,
            order: None,
        }
    }

    pub fn with_order(self, order: i32) -> Self {
        Self {
            order: Some(order),
            ..self
        }
    }

    pub fn order(&self) -> i32 {
        self.order.unwrap_or(0)
    }
}

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
    pub async fn add_record(
        &self,
        packet: &StorePacket<S>,
    ) -> StoreResult<DbRecord> {
        let db_record = packet
            .record
            .insert(&self.db, &packet.subject, packet.order())
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
            .update(&self.db, &packet.subject, packet.order())
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
            .upsert(&self.db, &packet.subject, packet.order())
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

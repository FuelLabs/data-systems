use std::sync::Arc;

use fuel_streams_macros::subject::IntoSubject;
use futures::stream::BoxStream;

use super::StoreError;
use crate::{
    db::Db,
    record::{Record, RecordPacket},
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct Store<S: Record> {
    pub db: Arc<Db>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record> Store<R> {
    pub fn new(db: &Arc<Db>) -> Self {
        Self {
            db: Arc::clone(db),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<R: Record> Store<R> {
    pub async fn insert_record(
        &self,
        packet: &RecordPacket<R>,
    ) -> StoreResult<R::DbItem> {
        let db_record = packet.record.insert(&self.db, packet).await?;
        Ok(db_record)
    }

    pub async fn find_many_by_subject(
        &self,
        subject: &Arc<dyn IntoSubject>,
        offset: i64,
        limit: i64,
    ) -> StoreResult<Vec<R::DbItem>> {
        R::find_many_by_subject(&self.db, subject, offset, limit)
            .await
            .map_err(StoreError::from)
    }

    pub async fn stream_by_subject(
        &self,
        subject: Arc<dyn IntoSubject>,
    ) -> StoreResult<BoxStream<'static, StoreResult<R::DbItem>>> {
        const DEFAULT_PAGE_SIZE: i64 = 100;
        let db = self.db.clone();

        let stream = async_stream::try_stream! {
            let mut offset = 0;
            loop {
                let items = R::find_many_by_subject(&db, &subject, offset, DEFAULT_PAGE_SIZE).await?;
                if items.is_empty() {
                    break;
                }

                for item in items {
                    yield item;
                }

                offset += DEFAULT_PAGE_SIZE;
            }
        };

        Ok(Box::pin(stream))
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn find_many_by_subject_ns(
        &self,
        subject: &Arc<dyn IntoSubject>,
        namespace: &str,
        offset: i64,
        limit: i64,
    ) -> StoreResult<Vec<R::DbItem>> {
        R::find_many_by_subject_ns(&self.db, subject, namespace, offset, limit)
            .await
            .map_err(StoreError::from)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn find_last_record_ns(
        &self,
        namespace: &str,
    ) -> StoreResult<Option<R::DbItem>> {
        R::find_last_record_ns(&self.db, namespace)
            .await
            .map_err(StoreError::from)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn stream_by_subject_ns(
        &self,
        subject: Arc<dyn IntoSubject>,
        namespace: String,
    ) -> StoreResult<BoxStream<'static, StoreResult<R::DbItem>>> {
        const DEFAULT_PAGE_SIZE: i64 = 100;
        let db = self.db.clone();

        let stream = async_stream::try_stream! {
            let mut offset = 0;
            loop {
                let items = R::find_many_by_subject_ns(&db, &subject, &namespace, offset, DEFAULT_PAGE_SIZE).await?;
                if items.is_empty() {
                    break;
                }

                for item in items {
                    yield item;
                }

                offset += DEFAULT_PAGE_SIZE;
            }
        };

        Ok(Box::pin(stream))
    }
}

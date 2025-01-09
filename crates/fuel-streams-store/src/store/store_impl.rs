use std::sync::Arc;

use fuel_streams_macros::subject::IntoSubject;
use futures::{stream, stream::BoxStream, StreamExt};

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
        let db_record = packet.record.insert(&self.db, &packet).await?;
        Ok(db_record)
    }

    pub async fn find_many_by_subject(
        &self,
        subject: &Arc<dyn IntoSubject>,
    ) -> StoreResult<Vec<R::DbItem>> {
        R::find_many_by_subject(&self.db, subject)
            .await
            .map_err(StoreError::from)
    }

    pub async fn stream_by_subject(
        &self,
        subject: &Arc<dyn IntoSubject>,
    ) -> StoreResult<BoxStream<'static, StoreResult<R::DbItem>>> {
        let items = self.find_many_by_subject(subject).await?;
        let stream = stream::iter(items)
            .map(|item| async move { Ok(item) })
            .buffered(10);

        Ok(Box::pin(stream))
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn find_many_by_subject_ns(
        &self,
        subject: &Arc<dyn IntoSubject>,
        namespace: &str,
    ) -> StoreResult<Vec<R::DbItem>> {
        R::find_many_by_subject_ns(&self.db, subject, namespace)
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
}

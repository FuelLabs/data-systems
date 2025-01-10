use std::sync::Arc;

use fuel_streams_macros::subject::IntoSubject;
use futures::stream::BoxStream;

use super::StoreError;
use crate::{
    db::Db,
    record::{QueryOptions, Record, RecordPacket},
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct Store<S: Record> {
    pub db: Arc<Db>,
    pub namespace: Option<String>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record> Store<R> {
    pub fn new(db: &Arc<Db>) -> Self {
        Self {
            db: Arc::clone(db),
            namespace: None,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace = Some(namespace.to_string());
        self
    }

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
        mut options: QueryOptions,
    ) -> StoreResult<Vec<R::DbItem>> {
        if cfg!(any(test, feature = "test-helpers")) {
            options = options.with_namespace(self.namespace.clone());
        }
        R::find_many_by_subject(&self.db, subject, options)
            .await
            .map_err(StoreError::from)
    }

    pub async fn stream_by_subject(
        &self,
        subject: Arc<dyn IntoSubject>,
        from_block: Option<u64>,
    ) -> StoreResult<BoxStream<'static, StoreResult<R::DbItem>>> {
        let db = self.db.clone();
        let namespace = self.namespace.clone();
        let stream = async_stream::try_stream! {
            let mut options = QueryOptions::default()
                .with_from_block(from_block)
                .with_namespace(namespace);
            loop {
                let items = R::find_many_by_subject(&db, &subject, options.clone()).await?;
                if items.is_empty() {
                    break;
                }
                for item in items {
                    yield item;
                }
                options.increment_offset();
            }
        };
        Ok(Box::pin(stream))
    }

    pub async fn find_last_record(&self) -> StoreResult<Option<R::DbItem>> {
        let namespace = if cfg!(any(test, feature = "test-helpers")) {
            self.namespace.as_deref()
        } else {
            None
        };

        R::find_last_record(&self.db, namespace)
            .await
            .map_err(StoreError::from)
    }
}

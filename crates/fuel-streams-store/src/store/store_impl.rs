use std::sync::Arc;

use fuel_data_parser::DataEncoder;
use fuel_streams_macros::subject::IntoSubject;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};

use super::{config, StoreError};
use crate::{
    db::Db,
    record::{DbTransaction, QueryOptions, Record, RecordPacket},
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct Store<S: Record + DataEncoder> {
    pub db: Arc<Db>,
    pub namespace: Option<String>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record + DataEncoder> Store<R> {
    pub fn new(db: &Arc<Db>) -> Self {
        Self {
            db: Arc::clone(db),
            namespace: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub async fn insert_record(
        &self,
        packet: &RecordPacket,
    ) -> StoreResult<R::DbItem> {
        let record = R::insert(&self.db.pool, packet).await?;
        Ok(record)
    }

    pub async fn insert_record_with_transaction(
        &self,
        tx: &mut DbTransaction,
        packet: &RecordPacket,
    ) -> StoreResult<R::DbItem> {
        let record = R::insert_with_transaction(tx, packet).await?;
        Ok(record)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn find_many_by_subject(
        &self,
        subject: &Arc<dyn IntoSubject>,
        mut options: QueryOptions,
    ) -> StoreResult<Vec<R::StoreItem>> {
        options = options.with_namespace(self.namespace.clone());
        let mut query =
            R::build_find_many_query(subject.clone(), options.clone());
        query
            .build_query_as::<R::StoreItem>()
            .fetch_all(&self.db.pool)
            .await
            .map_err(StoreError::from)
    }

    pub fn stream_by_subject(
        &self,
        subject: &Arc<dyn IntoSubject>,
        from_block: Option<u64>,
    ) -> BoxStream<'static, Result<R::StoreItem, StoreError>> {
        let db = Arc::clone(&self.db);
        let namespace = self.namespace.clone();
        let subject = subject.clone();
        async_stream::stream! {
            const MAX_BLOCK_RANGE: u64 = 100_000;
            let mut current_from_block = from_block;
            loop {
                let to_block = current_from_block.map(|b| b + MAX_BLOCK_RANGE);
                let options = QueryOptions::default()
                    .with_namespace(namespace.clone())
                    .with_from_block(current_from_block)
                    .with_to_block(to_block)
                    .with_limit(*config::STORE_PAGINATION_LIMIT);
                let mut query = R::build_find_many_query(subject.clone(), options.clone());
                let mut stream = query
                    .build_query_as::<R::StoreItem>()
                    .fetch(&db.pool);
                let mut found_records = false;
                while let Some(result) = stream.try_next().await? {
                    found_records = true;
                    yield Ok(result);
                }
                if !found_records || to_block.is_none() {
                    break;
                }
                current_from_block = to_block;
            }
        }
        .boxed()
    }

    pub async fn find_last_record(&self) -> StoreResult<Option<R::DbItem>> {
        let options =
            QueryOptions::default().with_namespace(self.namespace.clone());
        R::find_last_record(&self.db, options)
            .await
            .map_err(StoreError::from)
    }
}

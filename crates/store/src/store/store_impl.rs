use std::sync::Arc;

use fuel_data_parser::DataEncoder;
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::BlockHeight;
use futures::stream::BoxStream;

use super::StoreError;
use crate::{
    db::{Db, DbItem},
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

    pub async fn find_many_by_subject(
        &self,
        subject: &Arc<dyn fuel_streams_subject::subject::IntoSubject>,
        mut options: QueryOptions,
    ) -> StoreResult<Vec<R::DbItem>> {
        options = options.with_namespace(self.namespace.clone());
        let mut query =
            R::build_find_many_query(subject.clone(), options.clone());
        query
            .build_query_as::<R::DbItem>()
            .fetch_all(&self.db.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn historical_streaming(
        &self,
        subject: Arc<dyn IntoSubject>,
        from_block: Option<BlockHeight>,
        query_opts: Option<QueryOptions>,
    ) -> BoxStream<'static, Result<(String, Vec<u8>), StoreError>> {
        let store = self.clone();
        let db = self.db.clone();
        let stream = async_stream::try_stream! {
            let mut current_height = from_block.unwrap_or_default();
            let mut opts = query_opts.unwrap_or_default().with_from_block(Some(current_height));
            let mut last_height = find_last_block_height(&db, opts.clone()).await?;
            while current_height <= last_height {
                let items = store.find_many_by_subject(&subject, opts.clone()).await?;
                for item in items {
                    let subject = item.subject_str();
                    let value = item.encoded_value().to_vec();
                    yield (subject, value);
                    let block_height = item.get_block_height();
                    current_height = block_height;
                }
                opts.increment_offset();
                // When we reach the last known height, we need to check if any new blocks
                // were produced while we were processing the previous ones
                if current_height == last_height {
                    let new_last_height = find_last_block_height(&db, opts.clone()).await?;
                    if new_last_height > last_height {
                        // Reset current_height back to process the blocks we haven't seen yet
                        current_height = last_height;
                        last_height = new_last_height;
                    } else {
                        tracing::debug!("No new blocks found, stopping historical streaming on block {}", current_height);
                        break
                    }
                }
            }
        };
        Box::pin(stream)
    }
}

pub async fn find_last_block_height(
    db: &Db,
    options: QueryOptions,
) -> StoreResult<BlockHeight> {
    let select = "SELECT block_height FROM blocks".to_string();
    let mut query_builder = sqlx::QueryBuilder::new(select);
    if let Some(ns) = options.namespace {
        query_builder
            .push(" WHERE subject LIKE ")
            .push_bind(format!("{}%", ns));
    }

    query_builder.push(" ORDER BY block_height DESC LIMIT 1");
    let query = query_builder.build_query_as::<(i64,)>();
    let record: Option<(i64,)> = query
        .fetch_optional(&db.pool)
        .await
        .map_err(StoreError::FindLastBlockHeight)?;

    Ok(record.map(|(height,)| height.into()).unwrap_or_default())
}

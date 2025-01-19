use std::sync::Arc;

use fuel_data_parser::DataEncoder;

use super::StoreError;
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
        subject: &Arc<dyn fuel_streams_macros::subject::IntoSubject>,
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

    pub async fn find_last_record(&self) -> StoreResult<Option<R::DbItem>> {
        let options =
            QueryOptions::default().with_namespace(self.namespace.clone());
        R::find_last_record(&self.db, options)
            .await
            .map_err(StoreError::from)
    }
}

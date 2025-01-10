use std::sync::Arc;

use fuel_streams_macros::subject::IntoSubject;
use futures::{stream::BoxStream, StreamExt};

use super::{config, StoreError};
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
        with_retry(|| packet.record.insert(&self.db, packet)).await
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

    pub fn stream_by_subject(
        &self,
        subject: Arc<dyn IntoSubject>,
        from_block: Option<u64>,
    ) -> BoxStream<'static, Result<R::DbItem, StoreError>> {
        let db = Arc::clone(&self.db);
        async_stream::stream! {
            let options = QueryOptions::default()
                .with_from_block(from_block)
                .with_limit(*config::STORE_PAGINATION_LIMIT);

            let sql = R::build_find_many_query(subject, options);
            let mut stream = sqlx::query_as::<_, R::DbItem>(sql.as_str())
                .fetch(&db.pool);

            while let Some(result) = stream.next().await {
                yield result.map_err(StoreError::from);
            }
        }
        .boxed()
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

async fn with_retry<F, Fut, T, E>(f: F) -> StoreResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    StoreError: From<E>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                attempt += 1;
                if attempt >= *config::STORE_MAX_RETRIES {
                    return Err(StoreError::from(err));
                }

                // Exponential backoff: 100ms, 200ms, 400ms
                let initial_backoff_ms = *config::STORE_INITIAL_BACKOFF_MS;
                let delay = initial_backoff_ms * (1 << (attempt - 1));
                tokio::time::sleep(std::time::Duration::from_millis(delay))
                    .await;
            }
        }
    }
}

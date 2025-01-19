use std::{sync::Arc, time::Duration};

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_message_broker::MessageBroker;
use fuel_streams_domains::blocks::Block;
use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_store::{
    db::{Db, StoreItem},
    record::{QueryOptions, Record},
    store::Store,
};
use futures::{
    stream::{BoxStream, Stream as FStream},
    StreamExt,
};
use tokio::{sync::OnceCell, time::sleep};

use super::{config, StreamError};
use crate::server::DeliverPolicy;

pub type BoxedStoreItem = Result<(String, Vec<u8>), StreamError>;
pub type BoxedStream = Box<dyn FStream<Item = BoxedStoreItem> + Send + Unpin>;

#[derive(Debug, Clone)]
pub struct Stream<S: Record> {
    store: Arc<Store<S>>,
    broker: Arc<dyn MessageBroker>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record> Stream<R> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(
        broker: &Arc<dyn MessageBroker>,
        db: &Arc<Db>,
    ) -> Self {
        let cell = Self::INSTANCE;
        cell.get_or_init(|| async { Self::new(broker, db).await.to_owned() })
            .await
            .to_owned()
    }

    pub async fn new(broker: &Arc<dyn MessageBroker>, db: &Arc<Db>) -> Self {
        let store = Arc::new(Store::new(db));
        let broker = Arc::clone(broker);
        Self {
            store,
            broker,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn store(&self) -> &Store<R> {
        &self.store
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }

    pub async fn publish(
        &self,
        subject: &str,
        payload: bytes::Bytes,
    ) -> Result<(), StreamError> {
        let broker = self.broker.clone();
        broker.publish_event(subject, payload).await?;
        Ok(())
    }

    pub async fn subscribe_dynamic(
        &self,
        subject: Arc<dyn IntoSubject>,
        deliver_policy: DeliverPolicy,
    ) -> BoxStream<'static, Result<(String, Vec<u8>), StreamError>> {
        let db = self.store.db.clone();
        let broker = self.broker.clone();
        let subject = subject.clone();
        let stream = async_stream::try_stream! {
            if let DeliverPolicy::FromBlock { block_height } = deliver_policy {
                let mut historical = Self::historical_streaming(&db, subject.to_owned(), Some(block_height)).await;
                while let Some(result) = historical.next().await {
                    yield result?;
                    let throttle_time = *config::STREAM_THROTTLE_HISTORICAL;
                    sleep(Duration::from_millis(throttle_time as u64)).await;
                }
            }
            let mut live = broker.subscribe_to_events(&subject.parse()).await?;
            while let Some(msg) = live.next().await {
                yield msg?;
                let throttle_time = *config::STREAM_THROTTLE_LIVE;
                sleep(Duration::from_millis(throttle_time as u64)).await;
            }
        };
        Box::pin(stream)
    }

    pub async fn subscribe<S: IntoSubject>(
        &self,
        subject: S,
        deliver_policy: DeliverPolicy,
    ) -> BoxStream<'static, Result<(String, Vec<u8>), StreamError>> {
        let subject = Arc::new(subject);
        self.subscribe_dynamic(subject, deliver_policy).await
    }

    async fn historical_streaming(
        db: &Arc<Db>,
        subject: Arc<dyn IntoSubject>,
        from_block: Option<u64>,
    ) -> BoxStream<'static, Result<(String, Vec<u8>), StreamError>> {
        let db = db.clone();
        let stream = async_stream::try_stream! {
            let mut current_height = from_block.unwrap_or(0);
            let mut last_height = Self::find_last_block_height(&db).await?;
            while current_height <= last_height {
                let opts = QueryOptions::default().with_from_block(Some(current_height));
                let mut query = R::build_find_many_query(subject.to_owned(), opts.clone());
                let mut stream = query
                    .build_query_as::<R::StoreItem>()
                    .fetch(&db.pool);
                while let Some(result) = stream.next().await {
                    let result = result?;
                    let subject = result.subject_str();
                    let value = result.encoded_value().to_vec();
                    yield (subject, value);
                }
                current_height += 1;
                // When we reach the last known height, we need to check if any new blocks
                // were produced while we were processing the previous ones
                if current_height == last_height {
                    let new_last_height = Self::find_last_block_height(&db).await?;
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

    async fn find_last_block_height(db: &Arc<Db>) -> Result<u64, StreamError> {
        let opts = QueryOptions::default();
        let record = Block::find_last_record(db, opts).await?;
        match record {
            Some(record) => Ok(record.block_height as u64),
            None => Ok(0),
        }
    }
}

use std::{sync::Arc, time::Duration};

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_message_broker::MessageBroker;
use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_store::{
    db::{Db, DbItem},
    record::Record,
    store::Store,
};
use futures::{
    stream::{BoxStream, Stream as FStream},
    StreamExt,
};
use tokio::{sync::OnceCell, time::sleep};

use super::{config, StreamError};
use crate::DeliverPolicy;

pub type BoxedStreamItem = Result<Vec<u8>, StreamError>;
pub type BoxedStream = Box<dyn FStream<Item = BoxedStreamItem> + Send + Unpin>;

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
        db_record: &R::DbItem,
    ) -> Result<(), StreamError> {
        let broker = self.broker.clone();
        let encoded_value = db_record.encoded_value().to_vec();
        let subject = db_record.subject_str();
        broker.publish_event(&subject, encoded_value.into()).await?;
        Ok(())
    }

    pub async fn subscribe_dynamic(
        &self,
        subject: Arc<dyn IntoSubject>,
        deliver_policy: DeliverPolicy,
    ) -> BoxStream<'static, Result<Vec<u8>, StreamError>> {
        let store = self.store.clone();
        let broker = self.broker.clone();
        let subject_clone = subject.clone();
        let stream = async_stream::try_stream! {
            if let DeliverPolicy::FromBlock { block_height } = deliver_policy {
                let height = Some(block_height);
                let mut historical = store.stream_by_subject(&subject_clone, height);
                while let Some(result) = historical.next().await {
                    let item = result.map_err(StreamError::Store)?;
                    yield item.encoded_value().to_vec();
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
    ) -> BoxStream<'static, Result<Vec<u8>, StreamError>> {
        let subject = Arc::new(subject);
        self.subscribe_dynamic(subject, deliver_policy).await
    }
}

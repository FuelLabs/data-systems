use std::sync::Arc;

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_store::{
    db::{Db, DbItem},
    record::{Record, RecordPacket},
    store::Store,
};
use futures::{
    stream::{BoxStream, Stream as FStream},
    StreamExt,
};
use tokio::sync::OnceCell;

use super::StreamError;
use crate::{nats::*, DeliverPolicy};

pub type BoxedStreamItem = Result<(String, Vec<u8>), StreamError>;
pub type BoxedStream = Box<dyn FStream<Item = BoxedStreamItem> + Send + Unpin>;

#[derive(Debug, Clone)]
pub struct Stream<S: Record> {
    store: Arc<Store<S>>,
    nats_client: Arc<NatsClient>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record> Stream<R> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(nats_client: &NatsClient, db: &Arc<Db>) -> Self {
        let cell = Self::INSTANCE;
        cell.get_or_init(|| async {
            Self::new(nats_client, db).await.to_owned()
        })
        .await
        .to_owned()
    }

    pub async fn new(nats_client: &NatsClient, db: &Arc<Db>) -> Self {
        let store = Arc::new(Store::new(db));
        let nats_client = Arc::new(nats_client.clone());
        Self {
            store,
            nats_client,
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
        packet: &Arc<RecordPacket<R>>,
    ) -> Result<R::DbItem, StreamError> {
        let db_record = self.store.insert_record(packet).await?;
        self.publish_to_nats(packet.subject_str(), db_record.encoded_value())
            .await?;
        Ok(db_record)
    }

    async fn publish_to_nats(
        &self,
        subject: impl ToString,
        record_encoded: &[u8],
    ) -> Result<(), StreamError> {
        let client = self.nats_client.nats_client.clone();
        let payload = record_encoded.to_vec();
        client.publish(subject.to_string(), payload.into()).await?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        subject: Arc<dyn IntoSubject>,
        deliver_policy: DeliverPolicy,
    ) -> BoxStream<'static, Result<(String, Vec<u8>), StreamError>> {
        let store = self.store.clone();
        let client = self.nats_client.clone();
        let subject_clone = subject.clone();
        let stream = async_stream::try_stream! {
            if let DeliverPolicy::FromBlock { block_height } = deliver_policy {
                // Get historical data using store's built-in pagination
                let historical_stream = store
                    .stream_by_subject(subject_clone, Some(block_height))
                    .await?;
                futures::pin_mut!(historical_stream);
                while let Some(result) = historical_stream.next().await {
                    let item = result.map_err(StreamError::Store)?;
                    yield (item.subject_str(), item.encoded_value().to_vec());
                }
            }

            // Live subscription
            let client = client.nats_client.clone();
            let subscription = client.subscribe(subject.parse()).await?;
            let live_stream = subscription.then(|msg| async move {
                let payload = msg.payload;
                (msg.subject.to_string(), payload.to_vec())
            });

            futures::pin_mut!(live_stream);
            while let Some(msg) = live_stream.next().await {
                yield msg;
            }
        };

        Box::pin(stream)
    }
}

use std::sync::Arc;

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_store::{
    db::{Db, DbItem},
    record::{Record, RecordPacket},
    store::Store,
};
use futures::{
    stream::{BoxStream, Stream as FuturesStream, TryStreamExt},
    StreamExt,
};
use tokio::sync::OnceCell;

use super::StreamError;
use crate::nats::*;

pub type BoxedStream =
    Box<dyn FuturesStream<Item = (String, Vec<u8>)> + Send + Unpin>;

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

    pub async fn subscribe_live(
        &self,
        subject: &Arc<dyn IntoSubject>,
    ) -> Result<BoxStream<'static, (String, Vec<u8>)>, StreamError> {
        let client = self.nats_client.nats_client.clone();
        let subscriber = client.subscribe(subject.parse()).await?;
        let stream = subscriber.then(|msg| async move {
            let payload = msg.payload;
            (msg.subject.to_string(), payload.to_vec())
        });
        Ok(Box::pin(stream))
    }

    pub async fn subscribe_historical(
        &self,
        subject: Arc<dyn IntoSubject>,
    ) -> Result<BoxStream<'static, (String, Vec<u8>)>, StreamError> {
        let live_stream = self.subscribe_live(&subject).await?;
        let (live_sender, live_receiver) = tokio::sync::mpsc::channel(100);

        tokio::spawn({
            let mut live_stream = live_stream;
            async move {
                while let Some(result) = live_stream.next().await {
                    if live_sender.send(result).await.is_err() {
                        break;
                    }
                }
            }
        });

        let historical_stream = self.store.stream_by_subject(&subject).await?;
        let historical_stream = historical_stream
            .map_err(StreamError::Store)
            .map_ok(move |record| {
                (subject.parse(), record.encoded_value().to_vec())
            });

        let merged = async_stream::stream! {
            let mut historical = Box::pin(historical_stream);
            while let Some(result) = historical.next().await {
                if let Ok(record) = result {
                    yield record;
                }
            }
            let mut live_receiver = live_receiver;
            while let Some(record) = live_receiver.recv().await {
                yield record;
            }
        };

        Ok(Box::pin(merged))
    }
}

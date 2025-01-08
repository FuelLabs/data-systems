use std::sync::Arc;

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_streams_store::{
    db::{Db, DbRecord},
    record::{DataEncoder, Record},
    store::{Store, StoreError, StorePacket, StoreResult},
};
use futures::{
    stream::{BoxStream, TryStreamExt},
    StreamExt,
};
use tokio::sync::OnceCell;

use super::StreamError;
use crate::nats::*;

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
        packet: &StorePacket<R>,
    ) -> Result<DbRecord, StreamError> {
        let db_record = self.store.add_record(packet).await?;
        self.publish_to_nats(&db_record).await?;
        Ok(db_record)
    }

    async fn publish_to_nats(
        &self,
        db_record: &DbRecord,
    ) -> Result<(), StreamError> {
        let client = self.nats_client.nats_client.clone();
        let subject = db_record.subject.clone();
        let payload = db_record.encode().await?;
        client.publish(subject, payload.into()).await?;
        Ok(())
    }

    pub async fn subscribe_live(
        &self,
        subject: impl ToString,
    ) -> Result<BoxStream<'static, StoreResult<DbRecord>>, StreamError> {
        let client = self.nats_client.nats_client.clone();
        let subscriber = client.subscribe(subject.to_string()).await?;
        let stream = subscriber
            .then(|msg| async move {
                let payload = msg.payload;
                DbRecord::decode(&payload).await.map_err(StoreError::from)
            })
            .map(|result| result.map_err(StoreError::from));

        Ok(Box::pin(stream))
    }

    pub async fn subscribe_historical(
        &self,
        subject: impl Into<String>,
    ) -> Result<BoxStream<'static, DbRecord>, StreamError> {
        let subject = subject.into();
        let live_stream = self.subscribe_live(subject.clone()).await?;
        let (live_sender, live_receiver) = tokio::sync::mpsc::channel(100);

        tokio::spawn({
            let mut live_stream = live_stream;
            async move {
                while let Some(result) = live_stream.next().await {
                    if let Ok(record) = result {
                        if live_sender.send(record).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        let historical_stream =
            self.store.stream_by_subject_raw(&subject).await?;
        let historical_stream = historical_stream
            .map_err(StreamError::Store)
            .map_ok(|record| record);

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

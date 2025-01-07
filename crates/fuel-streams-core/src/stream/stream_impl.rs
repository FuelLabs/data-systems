use std::sync::{Arc, LazyLock};

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_streams_store::{
    db::{Db, DbRecord},
    record::Record,
    store::{Store, StorePacket},
};
use tokio::sync::OnceCell;

use super::StreamError;
use crate::nats::*;

pub static MAX_ACK_PENDING: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("MAX_ACK_PENDING")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(5)
});

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

    pub async fn publish(
        &self,
        packet: &StorePacket<R>,
    ) -> Result<DbRecord, StreamError> {
        let db_record = self.store.add_record(packet).await?;
        self.publish_to_nats(packet).await?;
        Ok(db_record)
    }

    async fn publish_to_nats(
        &self,
        packet: &StorePacket<R>,
    ) -> Result<(), StreamError> {
        let client = self.nats_client.nats_client.clone();
        let subject = packet.subject.clone();
        let payload = packet.record.encode().await?.into();
        client.publish(subject, payload).await?;
        Ok(())
    }

    pub async fn subscribe_live(
        &self,
        subject: impl ToString,
    ) -> Result<StreamLiveSubscriber, StreamError> {
        let client = self.nats_client.nats_client.clone();
        let subscriber = client.subscribe(subject.to_string()).await?;
        Ok(subscriber)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn store(&self) -> &Store<R> {
        &self.store
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }
}

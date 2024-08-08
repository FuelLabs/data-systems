use std::fmt::Debug;

use async_nats::jetstream::{
    consumer::{FromConsumer, IntoConsumerConfig},
    kv::Watch,
};
use async_trait::async_trait;
use fuel_streams_macros::subject::IntoSubject;

use super::{NatsClient, NatsNamespace, StoreError};
use crate::nats::{types::*, NatsError};

#[async_trait]
pub trait Storable: Debug + Clone + serde::Serialize {
    const STORE: &'static str;

    /// This is temporary until we have the data parser
    fn store_encode(&self) -> Result<Vec<u8>, StoreError> {
        bincode::serialize(self).map_err(StoreError::SerializationFailed)
    }

    /// This is temporary until we have the data parser
    fn store_decode<'a>(val: &'a [u8]) -> Result<Self, StoreError>
    where
        Self: serde::Deserialize<'a>,
    {
        bincode::deserialize(val).map_err(StoreError::SerializationFailed)
    }

    async fn create_store(
        client: &NatsClient,
    ) -> Result<Store<Self>, NatsError> {
        let nats_store = client.create_store(Self::STORE, None).await?;
        Ok(Store::new(nats_store, &client.namespace))
    }
}

#[derive(Debug, Clone)]
pub struct Store<S: Storable> {
    pub store: NatsStore,
    namespace: NatsNamespace,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Storable> Store<S> {
    pub fn new(store: NatsStore, namespace: &NatsNamespace) -> Self {
        Self {
            store,
            namespace: namespace.to_owned(),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn upsert(
        &self,
        subject: &impl IntoSubject,
        payload: &S,
    ) -> Result<u64, StoreError> {
        let key = self.subject_name(&subject.parse());
        self.store
            .put(key.to_owned(), payload.store_encode()?.into())
            .await
            .map_err(|s| StoreError::UpsertFailed { key, source: s })
    }

    pub async fn watch(&self, key: &str) -> Result<Watch, StoreError> {
        let subject = self.namespace.subject_name(key);
        self.store
            .watch(subject.clone())
            .await
            .map_err(|source| StoreError::WatchFailed { subject, source })
    }

    pub async fn create_consumer<
        C: FromConsumer + IntoConsumerConfig + Default,
    >(
        &self,
        config: C,
    ) -> Result<NatsConsumer<C>, StoreError> {
        let stream = self.store.stream.clone();
        stream
            .create_consumer(config)
            .await
            .map_err(StoreError::CreateConsumerFailed)
    }

    pub fn subject_name(&self, val: &str) -> String {
        self.namespace.subject_name(val)
    }
}

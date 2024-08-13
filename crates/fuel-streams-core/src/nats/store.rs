use std::fmt::Debug;

use async_nats::jetstream::{
    consumer::{FromConsumer, IntoConsumerConfig},
    kv::Watch,
};
use async_trait::async_trait;
use fuel_data_parser::{
    DataParser,
    DataParserDeserializable,
    DataParserSerializable,
    NatsFormattedMessage,
};
use fuel_streams_macros::subject::IntoSubject;

use super::{NatsClient, NatsNamespace, StoreError};
use crate::nats::{types::*, NatsError};

#[async_trait]
pub trait Storable: DataParserSerializable {
    const STORE: &'static str;

    /// This is temporary until we have the data parser
    async fn store_encode(
        &self,
        subject: &impl IntoSubject,
        data_parser: &DataParser,
    ) -> Result<Vec<u8>, StoreError> {
        data_parser
            .to_nats_payload(subject, self)
            .await
            .map_err(StoreError::DataParser)
    }

    /// This is temporary until we have the data parser
    async fn store_decode<T: DataParserDeserializable>(
        val: Vec<u8>,
        data_parser: &DataParser,
    ) -> Result<NatsFormattedMessage<T>, StoreError> {
        Ok(data_parser
            .from_nats_message(val)
            .await
            .map_err(StoreError::DataParser)?)
    }

    async fn create_store(
        client: &NatsClient,
        data_parser: &DataParser,
    ) -> Result<Store<Self>, NatsError> {
        let nats_store = client.create_store(Self::STORE, None).await?;
        Ok(Store::new(nats_store, &client.namespace, data_parser))
    }
}

#[derive(Debug, Clone)]
pub struct Store<S: Storable> {
    pub store: NatsStore,
    namespace: NatsNamespace,
    data_parser: DataParser,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Storable + Send + Sync> Store<S> {
    pub fn new(
        store: NatsStore,
        namespace: &NatsNamespace,
        data_parser: &DataParser,
    ) -> Self {
        Self {
            store,
            namespace: namespace.to_owned(),
            data_parser: data_parser.clone(),
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
            .put(
                key.to_owned(),
                payload
                    .store_encode(subject, &self.data_parser)
                    .await?
                    .into(),
            )
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

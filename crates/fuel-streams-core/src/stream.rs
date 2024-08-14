use std::fmt::Debug;

use async_nats::jetstream::stream::DirectGetErrorKind;
use async_trait::async_trait;
use fuel_streams_macros::subject::IntoSubject;

use crate::nats::{
    types::*,
    NatsClient,
    NatsError,
    NatsNamespace,
    StreamError,
};

/// Houses nats-agnostic APIs for making a fuel-core type streamable
#[async_trait]
pub trait Streamable: Debug + Clone + serde::Serialize {
    const STORE: &'static str;

    /// This is temporary until we have the data parser
    fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Streamable must be serializable")
    }

    /// This is temporary until we have the data parser
    fn decode(val: &[u8]) -> Self
    where
        Self: for<'de> serde::Deserialize<'de>,
    {
        bincode::deserialize(val).expect("Streamable must be deserializable")
    }

    async fn create_stream(
        client: &NatsClient,
    ) -> Result<Stream<Self>, NatsError> {
        let nats_kv_store = client.create_kv_store(Self::STORE, None).await?;
        Ok(Stream::new(nats_kv_store, &client.namespace))
    }
}

/// Houses nats-agnostic APIs for publishing and consuming a streamable type
#[derive(Debug, Clone)]
pub struct Stream<S: Streamable> {
    pub store: NatsStore,
    namespace: NatsNamespace,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Streamable> Stream<S> {
    fn subject_name(&self, val: &str) -> String {
        self.namespace.subject_name(val)
    }

    fn new(store: NatsStore, namespace: &NatsNamespace) -> Self {
        Self {
            store,
            namespace: namespace.to_owned(),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn publish(
        &self,
        subject: &impl IntoSubject,
        payload: &S,
    ) -> Result<u64, StreamError> {
        let subject_name = &self.subject_name(&subject.parse());
        self.store
            .put(subject_name.to_owned(), payload.encode().into())
            .await
            .map_err(|s| StreamError::PublishFailed {
                subject_name: subject_name.to_string(),
                source: s,
            })
    }

    pub async fn get_last_published(
        &self,
        wildcard: &'static str,
    ) -> Result<Option<S>, StreamError>
    where
        S: for<'de> serde::Deserialize<'de>,
    {
        let subject_name = &self.subject_name(wildcard);
        let message = self
            .store
            .stream
            .direct_get_last_for_subject(subject_name)
            .await;

        match message {
            Ok(message) => {
                let payload = S::decode(&message.payload);

                Ok(Some(payload))
            }
            Err(error) => match &error.kind() {
                DirectGetErrorKind::NotFound => Ok(None),
                _ => Err(StreamError::GetLastPublishedFailed {
                    subject_name: subject_name.to_string(),
                    source: error,
                }),
            },
        }
    }

    pub async fn subscribe(
        &self,
        key: &str,
    ) -> Result<impl futures_util::Stream, StreamError> {
        let subject_name = &self.namespace.subject_name(key);

        self.store.watch(&subject_name).await.map_err(|source| {
            StreamError::SubscriptionFailed {
                subject_name: subject_name.to_string(),
                source,
            }
        })
    }
}

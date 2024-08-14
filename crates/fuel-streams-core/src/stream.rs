use std::fmt::Debug;

use async_nats::{jetstream::stream::LastRawMessageErrorKind, Message};
use async_trait::async_trait;
use fuel_data_parser::DataParser;
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
pub trait Streamable: Debug + Clone + serde::Serialize + Send + Sync {
    const STORE: &'static str;

    async fn encode(&self, subject: &str) -> Vec<u8> {
        Self::data_parser()
            .to_nats_payload(subject, self)
            .await
            .expect("Streamable must be encode correctly")
    }

    async fn decode(bytes: &[u8]) -> Self
    where
        Self: for<'de> serde::Deserialize<'de>,
    {
        let message = Self::data_parser()
            .from_nats_message(bytes.to_vec())
            .await
            .expect("Streamable must be decode correctly");

        message.data
    }

    async fn create_stream(
        client: &NatsClient,
    ) -> Result<Stream<Self>, NatsError> {
        let nats_kv_store = client.create_kv_store(Self::STORE, None).await?;
        Ok(Stream::new(nats_kv_store, &client.namespace))
    }

    fn data_parser() -> DataParser {
        DataParser::default()
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
            .put(
                subject_name.to_owned(),
                payload.encode(subject_name).await.into(),
            )
            .await
            .map_err(|s| StreamError::PublishFailed {
                subject_name: subject_name.to_string(),
                source: s,
            })
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

    #[cfg(feature = "test-helpers")]
    pub async fn is_empty(&self, wildcard: &str) -> bool
    where
        S: for<'de> serde::Deserialize<'de>,
    {
        self.get_last_published(wildcard)
            .await
            .is_ok_and(|result| result.is_none())
    }

    /// TODO: investigate why this always returns None even after publishing (putting to the KV Store)
    pub async fn get_last_published(
        &self,
        wildcard: &str,
    ) -> Result<Option<S>, StreamError>
    where
        S: for<'de> serde::Deserialize<'de>,
    {
        let wildcard = &self.subject_name(wildcard);

        // An hack to ensure we keep the KV namespace when interacting
        // with the KV store's stream
        let subject_name = &format!("$KV.*.{wildcard}");

        let message = self
            .store
            .stream
            .get_last_raw_message_by_subject(subject_name)
            .await;

        match message {
            Ok(message) => {
                let message: Message = message.try_into().unwrap();
                let payload = S::decode(&message.payload).await;

                Ok(Some(payload))
            }
            Err(error) => match &error.kind() {
                LastRawMessageErrorKind::NoMessageFound => Ok(None),
                _ => Err(StreamError::GetLastPublishedFailed {
                    subject_name: subject_name.to_string(),
                    source: error,
                }),
            },
        }
    }
}

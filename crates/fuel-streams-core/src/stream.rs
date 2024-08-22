mod error;

use std::fmt::Debug;

use async_nats::{
    jetstream::{
        consumer::AckPolicy,
        kv,
        stream::{self, LastRawMessageErrorKind},
    },
    Message,
};
use async_trait::async_trait;
pub use error::StreamError;
use fuel_data_parser::{
    DataParser,
    DataParserDeserializable,
    DataParserSerializable,
    NatsFormattedMessage,
};
use fuel_streams_macros::subject::IntoSubject;
use futures::StreamExt;
use tokio::sync::OnceCell;

use crate::{nats::types::*, prelude::NatsClient};

#[async_trait]
pub trait StreamEncoder:
    Debug
    + Clone
    + Send
    + Sync
    + DataParserSerializable
    + DataParserDeserializable
    + 'static
{
    async fn encode(&self, subject: &str) -> Vec<u8> {
        Self::data_parser()
            .to_nats_payload(subject, self)
            .await
            .expect("Streamable must encode correctly")
    }

    async fn decode(encoded: Vec<u8>) -> Self {
        Self::decode_raw(encoded).await.data
    }

    async fn decode_raw(encoded: Vec<u8>) -> NatsFormattedMessage<Self> {
        Self::data_parser()
            .from_nats_message(encoded)
            .await
            .expect("Streamable must decode correctly")
    }

    fn data_parser() -> DataParser {
        DataParser::default()
    }
}

#[async_trait]
pub trait Streamable: StreamEncoder {
    const NAME: &'static str;
    const WILDCARD_LIST: &'static [&'static str];
}

/// Houses nats-agnostic APIs for publishing and consuming a streamable type
/// TODO: Split this into two traits StreamPublisher + StreamSubscriber
#[derive(Debug, Clone)]
pub struct Stream<S: Streamable> {
    store: kv::Store,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Streamable> Stream<S> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(client: &NatsClient) -> Self {
        let cell = Self::INSTANCE;
        cell.get_or_init(|| async { Self::new(client).await.to_owned() })
            .await
            .to_owned()
    }

    pub async fn new(client: &NatsClient) -> Self {
        let namespace = &client.namespace;
        let bucket_name = namespace.stream_name(S::NAME);

        let store = client
            .get_or_create_kv_store(kv::Config {
                bucket: bucket_name.to_owned(),
                storage: stream::StorageType::File,
                compression: true,
                ..Default::default()
            })
            .await
            .expect("Streams must be created");

        Self {
            store,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn publish(
        &self,
        subject: &impl IntoSubject,
        payload: &S,
    ) -> Result<u64, StreamError> {
        let subject_name = &subject.parse();
        self.store
            .put(subject_name, payload.encode(subject_name).await.into())
            .await
            .map_err(|s| StreamError::PublishFailed {
                subject_name: subject_name.to_string(),
                source: s,
            })
    }

    pub async fn subscribe(
        &self,
        // TODO: Allow encapsulating Subject to return wildcard token type
        wildcard: &str,
    ) -> Result<impl futures::Stream<Item = Vec<u8>>, StreamError> {
        Ok(self
            .store
            .watch(&wildcard)
            .await
            .map(|stream| stream.map(|entry| entry.unwrap().value.to_vec()))?)
    }

    // TODO: Make this interface more Stream-like and Nats agnostic
    pub async fn subscribe_consumer(
        &self,
        config: SubscribeConsumerConfig,
    ) -> Result<PullConsumerStream, StreamError> {
        let config = PullConsumerConfig {
            filter_subjects: config.filter_subjects,
            deliver_policy: config.deliver_policy,
            ack_policy: AckPolicy::None,
            ..Default::default()
        };

        let config = self.prefix_filter_subjects(config);
        let consumer = self.store.stream.create_consumer(config).await?;
        Ok(consumer.messages().await?)
    }

    // TODO: Make this interface more Stream-like and Nats agnostic
    pub async fn create_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<NatsConsumer<PullConsumerConfig>, StreamError> {
        let config = self.prefix_filter_subjects(config);
        Ok(self.store.stream.create_consumer(config).await?)
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
    ) -> Result<Option<S>, StreamError> {
        let subject_name = &Self::prefix_filter_subject(wildcard);

        let message = self
            .store
            .stream
            .get_last_raw_message_by_subject(subject_name)
            .await;

        match message {
            Ok(message) => {
                let message: Message = message.try_into().unwrap();
                let payload = S::decode(message.payload.to_vec()).await;

                Ok(Some(payload))
            }
            Err(error) => match &error.kind() {
                LastRawMessageErrorKind::NoMessageFound => Ok(None),
                _ => Err(error.into()),
            },
        }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn assert_has_stream(
        &self,
        names: &std::collections::HashSet<String>,
    ) {
        let mut stream = self.store.stream.clone();
        let info = stream.info().await.unwrap();
        let has_stream = names.iter().any(|n| n.eq(&info.config.name));
        assert!(has_stream)
    }

    fn prefix_filter_subjects(
        &self,
        mut config: PullConsumerConfig,
    ) -> PullConsumerConfig {
        config.filter_subjects = config
            .filter_subjects
            .iter()
            .map(Self::prefix_filter_subject)
            .collect();
        config
    }

    fn prefix_filter_subject(subject: impl Into<String>) -> String {
        // An hack to ensure we keep the KV namespace when reading
        // from the KV store's stream
        let subject = subject.into();
        format!("$KV.*.{subject}")
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubscribeConsumerConfig {
    pub filter_subjects: Vec<String>,
    pub deliver_policy: DeliverPolicy,
}

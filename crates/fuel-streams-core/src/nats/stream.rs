use std::fmt::Debug;

use async_nats::jetstream::{
    consumer::AckPolicy,
    context::Publish,
    kv,
    stream,
};
use async_trait::async_trait;
use fuel_data_parser::{
    DataParser,
    DataParserDeserializable,
    DataParserSerializable,
    NatsFormattedMessage,
};
use fuel_streams_macros::subject::IntoSubject;
use tokio::sync::OnceCell;

use super::{NatsError, NatsNamespace};
use crate::{nats::types::*, prelude::NatsClient};

// ------------------------------------------------------------------------
// Traits
// ------------------------------------------------------------------------

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
    async fn stream_encode(
        &self,
        subject: &str,
    ) -> Result<bytes::Bytes, NatsError> {
        let data_parser = DataParser::default();
        let encoded = data_parser.to_nats_payload(subject, self).await?;
        Ok(encoded.into())
    }

    async fn stream_decode(
        encoded: Vec<u8>,
    ) -> Result<NatsFormattedMessage<Self>, NatsError> {
        let data_parser = DataParser::default();
        Ok(data_parser.from_nats_message(encoded).await?)
    }
}

#[async_trait]
pub trait Streamable:
    Debug + Clone + Send + Sync + Sized + StreamEncoder
{
    const NAME: &'static str;
    const WILDCARD_LIST: &'static [&'static str];

    type Builder: StreamItem<Self> + Send + Sync;
    type MainSubject: IntoSubject + Send + Sync;
}

#[async_trait]
pub trait StreamItem<S: Streamable + StreamEncoder>:
    Debug + Clone + Send + Sync
{
    type Config: Send + Sync;
    type Subscriber;

    fn nats_stream(&self) -> &stream::Stream;

    fn name(ns: &NatsNamespace) -> String {
        ns.stream_name(S::NAME)
    }

    async fn new(
        client: &NatsClient,
        config: Option<Self::Config>,
    ) -> Result<Self, NatsError>;

    async fn publish(
        &self,
        client: &NatsClient,
        subject: &str,
        payload: bytes::Bytes,
    ) -> Result<(), NatsError>;

    async fn subscribe(
        &self,
        subject: &str,
    ) -> Result<Self::Subscriber, NatsError>;

    async fn subscribe_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<PullConsumerStream, NatsError>;

    async fn create_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError>;
}

// ------------------------------------------------------------------------
// NatsStore
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NatsStore<S: Streamable + StreamEncoder> {
    pub store: kv::Store,
    _marker: std::marker::PhantomData<S>,
}

#[async_trait]
impl<S: Streamable + StreamEncoder> StreamItem<S> for NatsStore<S> {
    type Config = ();
    type Subscriber = kv::Watch;

    fn nats_stream(&self) -> &stream::Stream {
        &self.store.stream
    }

    async fn new(
        client: &NatsClient,
        _: Option<Self::Config>,
    ) -> Result<Self, NatsError> {
        let name = Self::name(&client.namespace);
        let store = client
            .get_or_create_store(kv::Config {
                bucket: name.to_owned(),
                storage: stream::StorageType::File,
                compression: true,
                ..Default::default()
            })
            .await?;

        Ok(Self {
            store,
            _marker: std::marker::PhantomData,
        })
    }

    async fn publish(
        &self,
        _client: &NatsClient,
        subject: &str,
        payload: bytes::Bytes,
    ) -> Result<(), NatsError> {
        self.store.put(subject.to_owned(), payload).await?;
        Ok(())
    }

    async fn subscribe(
        &self,
        subject: &str,
    ) -> Result<Self::Subscriber, NatsError> {
        self.store
            .watch(subject)
            .await
            .map_err(NatsError::StoreSubscribe)
    }

    async fn subscribe_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<PullConsumerStream, NatsError> {
        let config = self.prefix_filter_subjects(config);
        let consumer = self.nats_stream().create_consumer(config).await?;
        Ok(consumer.messages().await?)
    }

    async fn create_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        let config = self.prefix_filter_subjects(config);
        Ok(self.nats_stream().create_consumer(config).await?)
    }
}

impl<S: Streamable + StreamEncoder> NatsStore<S> {
    fn prefix_filter_subjects(
        &self,
        mut config: PullConsumerConfig,
    ) -> PullConsumerConfig {
        config.filter_subjects = config
            .filter_subjects
            .iter()
            .map(|s| format!("$KV.*.{s}"))
            .collect();
        config
    }
}

// ------------------------------------------------------------------------
// NatsStream
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NatsStream<S: Streamable + StreamEncoder> {
    pub stream: stream::Stream,
    pub namespace: NatsNamespace,
    _marker: std::marker::PhantomData<S>,
}

#[async_trait]
impl<S: Streamable + StreamEncoder> StreamItem<S> for NatsStream<S> {
    type Config = stream::Config;
    type Subscriber = NatsConsumer<PullConsumerConfig>;

    fn nats_stream(&self) -> &stream::Stream {
        &self.stream
    }

    async fn new(
        client: &NatsClient,
        options: Option<Self::Config>,
    ) -> Result<Self, NatsError> {
        let name = Self::name(&client.namespace);
        let stream = client
            .get_or_create_stream(stream::Config {
                name,
                subjects: Self::subject_list(client),
                ..options.unwrap_or_default()
            })
            .await?;

        Ok(Self {
            stream,
            namespace: client.namespace.to_owned(),
            _marker: std::marker::PhantomData,
        })
    }

    async fn publish(
        &self,
        client: &NatsClient,
        subject: &str,
        payload: bytes::Bytes,
    ) -> Result<(), NatsError> {
        let subject = self.prefix_subject(subject);
        let publish = Publish::build().message_id(&subject).payload(payload);
        client
            .jetstream
            .send_publish(subject, publish)
            .await?
            .await?;
        Ok(())
    }

    async fn subscribe(
        &self,
        subject: &str,
    ) -> Result<Self::Subscriber, NatsError> {
        self.stream
            .create_consumer(PullConsumerConfig {
                filter_subject: self.prefix_subject(subject),
                deliver_policy: DeliverPolicy::New,
                ack_policy: AckPolicy::None,
                ..Default::default()
            })
            .await
            .map_err(NatsError::ConsumerCreate)
    }

    async fn subscribe_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<PullConsumerStream, NatsError> {
        let config = self.prefix_filter_subjects(config);
        let consumer = self.nats_stream().create_consumer(config).await?;
        Ok(consumer.messages().await?)
    }

    async fn create_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        let config = self.prefix_filter_subjects(config);
        Ok(self.nats_stream().create_consumer(config).await?)
    }
}

impl<S: Streamable> NatsStream<S> {
    fn subject_list(client: &NatsClient) -> Vec<String> {
        S::WILDCARD_LIST
            .iter()
            .map(|s| client.namespace.subject_name(s))
            .collect()
    }

    fn prefix_subject(&self, val: &str) -> String {
        self.namespace.subject_name(val)
    }

    fn prefix_filter_subjects(
        &self,
        mut config: PullConsumerConfig,
    ) -> PullConsumerConfig {
        config.filter_subjects = config
            .filter_subjects
            .iter()
            .map(|s| self.prefix_subject(s))
            .collect();
        config
    }
}

// ------------------------------------------------------------------------
// Streamer
// ------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct SubscribeConsumerConfig {
    pub filter_subjects: Vec<String>,
    pub deliver_policy: DeliverPolicy,
}

#[derive(Debug, Clone)]
pub struct Streamer<S: Streamable + StreamEncoder> {
    item: S::Builder,
    client: NatsClient,
}

impl<S: Streamable + StreamEncoder> Streamer<S> {
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(
        client: &NatsClient,
        options: Option<<S::Builder as StreamItem<S>>::Config>,
    ) -> Result<Self, NatsError> {
        let cell = Self::INSTANCE;
        let stream = cell
            .get_or_init(|| async {
                let item =
                    S::Builder::new(client, options).await.unwrap().to_owned();
                Self {
                    item,
                    client: client.to_owned(),
                }
            })
            .await
            .to_owned();
        Ok(stream)
    }

    pub async fn publish(
        &self,
        subject: &str,
        payload: &S,
    ) -> Result<(), NatsError> {
        let encoded = payload.stream_encode(subject).await?;
        self.item.publish(&self.client, subject, encoded).await?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        subject: &str,
    ) -> Result<<S::Builder as StreamItem<S>>::Subscriber, NatsError> {
        self.item.subscribe(subject).await
    }

    pub async fn subscribe_consumer(
        &self,
        config: SubscribeConsumerConfig,
    ) -> Result<PullConsumerStream, NatsError> {
        let config = PullConsumerConfig {
            filter_subjects: config.filter_subjects,
            deliver_policy: config.deliver_policy,
            ack_policy: AckPolicy::None,
            ..Default::default()
        };
        self.item.subscribe_consumer(config).await
    }

    pub async fn create_consumer(
        &self,
        config: PullConsumerConfig,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        self.item.create_consumer(config).await
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn assert_has_nats_stream(
        &self,
        names: &std::collections::HashSet<String>,
    ) {
        let mut stream = self.item.nats_stream().clone();
        let info = stream.info().await.unwrap();
        let has_stream = names.iter().any(|n| n.eq(&info.config.name));
        assert!(has_stream)
    }
}

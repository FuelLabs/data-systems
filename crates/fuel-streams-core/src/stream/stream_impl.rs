#[cfg(any(test, feature = "test-helpers"))]
use std::pin::Pin;
use std::{fmt::Debug, sync::Arc};

use async_nats::{
    jetstream::{
        consumer::AckPolicy,
        kv::{self, CreateErrorKind},
        stream::{self, LastRawMessageErrorKind, State},
    },
    RequestErrorKind,
};
use async_trait::async_trait;
use fuel_networks::FuelNetwork;
use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_storage::*;
use futures::{StreamExt, TryStreamExt};
use sha2::{Digest, Sha256};
use tokio::sync::OnceCell;

use super::{error::StreamError, stream_encoding::StreamEncoder};

#[derive(Clone)]
pub struct PublishPacket<T: Streamable> {
    pub subject: Arc<dyn IntoSubject>,
    pub payload: Arc<T>,
    pub s3_path: String,
}

impl<T: Streamable> PublishPacket<T> {
    pub fn new(payload: T, subject: Arc<dyn IntoSubject>) -> Self {
        let s3_path = payload.get_s3_path(&subject.parse());

        Self {
            payload: Arc::new(payload),
            subject,
            s3_path,
        }
    }
}

/// Trait for types that can be streamed.
///
/// # Examples
///
/// ```no_run
/// use async_trait::async_trait;
/// use fuel_streams_core::prelude::*;
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// struct MyStreamable {
///     data: String,
/// }
///
/// impl StreamEncoder for MyStreamable {}
///
/// #[async_trait]
/// impl Streamable for MyStreamable {
///     const NAME: &'static str = "my_streamable";
///     const WILDCARD_LIST: &'static [&'static str] = &["*"];
/// }
/// ```
#[async_trait]
pub trait Streamable: StreamEncoder {
    const NAME: &'static str;
    const WILDCARD_LIST: &'static [&'static str];

    fn to_packet(&self, subject: Arc<dyn IntoSubject>) -> PublishPacket<Self> {
        PublishPacket::new(self.clone(), subject)
    }

    fn get_s3_path(&self, subject: &str) -> String {
        format!(
            "{}/{}/v1/{}.json",
            FuelNetwork::load_from_env(),
            Self::NAME,
            self.get_consistent_hash(subject)
        )
    }

    fn get_consistent_hash(&self, subject: &str) -> String {
        let serialized = self.encode(subject);

        let mut hasher = Sha256::new();
        hasher.update(serialized);

        format!("{:x}", hasher.finalize())
    }
}

/// Houses nats-agnostic APIs for publishing and consuming a streamable type
///
/// # Examples
///
/// ```no_run
/// use fuel_streams_core::prelude::*;
/// use fuel_streams_macros::subject::IntoSubject;
/// use futures::StreamExt;
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// struct MyStreamable {
///     data: String,
/// }
///
/// impl StreamEncoder for MyStreamable {}
///
/// #[async_trait::async_trait]
/// impl Streamable for MyStreamable {
///     const NAME: &'static str = "my_streamable";
///     const WILDCARD_LIST: &'static [&'static str] = &["*"];
/// }
///
/// async fn example(client: &NatsClient) {
///     let stream = Stream::<MyStreamable>::new(client).await;
///
///     // Publish
///     let subject = BlocksSubject::new().with_height(Some(23.into()));
///     let payload = MyStreamable { data: "foo".into() };
///     stream.publish(&subject, &payload).await.unwrap();
///
///     // Subscribe
///     let wildcard = BlocksSubject::WILDCARD;
///     let mut subscription = stream.subscribe(wildcard).await.unwrap();
///     while let Some(message) = subscription.next().await {
///         // Process message
///     }
/// }
/// ```
///
/// TODO: Split this into two traits StreamPublisher + StreamSubscriber
/// TODO: Rename as FuelStream?
#[derive(Debug, Clone)]
pub struct Stream<S: Streamable> {
    store: kv::Store,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Streamable> Stream<S> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(nats_client: &NatsClient) -> Self {
        let cell = Self::INSTANCE;
        cell.get_or_init(|| async { Self::new(nats_client).await.to_owned() })
            .await
            .to_owned()
    }

    pub async fn new(nats_client: &NatsClient) -> Self {
        let namespace = &nats_client.namespace;
        let bucket_name = namespace.stream_name(S::NAME);
        let store = nats_client
            .get_or_create_kv_store(kv::Config {
                bucket: bucket_name.to_owned(),
                storage: stream::StorageType::File,
                history: 1,
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
        packet: &PublishPacket<S>,
        s3_client: &S3Client,
    ) -> Result<usize, StreamError> {
        let payload = &packet.payload;
        let s3_path = &packet.s3_path;
        let subject_name = &packet.subject.parse();

        // publish payload to S3
        s3_client
            .put_object(s3_path, payload.encode(subject_name))
            .await?;

        self.publish_s3_path_to_nats(subject_name, s3_path).await
    }

    async fn publish_s3_path_to_nats(
        &self,
        subject_name: &str,
        s3_path: &str,
    ) -> Result<usize, StreamError> {
        let data = s3_path.to_string().into_bytes();
        let data_size = data.len();

        let result = self.store.create(subject_name, data.into()).await;

        match result {
            Ok(_) => Ok(data_size),
            Err(e) if e.kind() == CreateErrorKind::AlreadyExists => {
                Ok(data_size)
            }
            Err(e) => Err(StreamError::PublishFailed {
                subject_name: subject_name.to_string(),
                source: e,
            }),
        }
    }

    pub async fn get_consumers_and_state(
        &self,
    ) -> Result<(String, Vec<String>, State), RequestErrorKind> {
        let mut consumers = vec![];
        while let Ok(Some(consumer)) =
            self.store.stream.consumer_names().try_next().await
        {
            consumers.push(consumer);
        }

        let state = self.store.stream.cached_info().state.clone();
        let stream_name = self.get_stream_name().to_string();
        Ok((stream_name, consumers, state))
    }

    pub fn get_stream_name(&self) -> &str {
        self.store.stream_name.as_str()
    }

    // TODO: This should probably be `subscribe_raw` since it returns pure bytes
    pub async fn subscribe(
        &self,
        // TODO: Allow encapsulating Subject to return wildcard token type
        wildcard: &str,
    ) -> Result<impl futures::Stream<Item = Option<Vec<u8>>>, StreamError> {
        Ok(self.store.watch(&wildcard).await.map(|stream| {
            stream.map(|entry| {
                entry.ok().map(|entry_item| entry_item.value.to_vec())
            })
        })?)
    }

    #[cfg(feature = "test-helpers")]
    /// Fetch all old messages from this stream
    pub async fn catchup(
        &self,
        number_of_messages: usize,
    ) -> Result<
        Pin<Box<dyn futures::Stream<Item = Option<S>> + Send>>,
        StreamError,
    > {
        let config = PullConsumerConfig {
            filter_subjects: self.all_filter_subjects(),
            deliver_policy: DeliverPolicy::All,
            ack_policy: AckPolicy::None,
            ..Default::default()
        };
        let config = self.prefix_filter_subjects(config);
        let consumer = self.store.stream.create_consumer(config).await?;

        let stream = consumer.messages().await?.take(number_of_messages).then(
            |message| async {
                if let Ok(message) = message {
                    Some(S::decode(message.payload.to_vec()))
                } else {
                    None
                }
            },
        );

        // Use Box::pin to pin the stream on the heap
        Ok(Box::pin(stream))
    }

    // TODO: Make this interface more Stream-like and Nats agnostic
    // TODO: This should probably be removed in favor of `subscribe`
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
    fn all_filter_subjects(&self) -> Vec<String> {
        S::WILDCARD_LIST.iter().map(|s| s.to_string()).collect()
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
                let payload = S::decode(message.payload.to_vec());

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

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn store(&self) -> &kv::Store {
        &self.store
    }
}

/// Configuration for subscribing to a consumer.
///
/// # Examples
///
/// ```
/// use fuel_streams_core::stream::SubscribeConsumerConfig;
/// use async_nats::jetstream::consumer::DeliverPolicy;
///
/// let config = SubscribeConsumerConfig {
///     filter_subjects: vec!["example.*".to_string()],
///     deliver_policy: DeliverPolicy::All,
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct SubscribeConsumerConfig {
    pub filter_subjects: Vec<String>,
    pub deliver_policy: DeliverPolicy,
}

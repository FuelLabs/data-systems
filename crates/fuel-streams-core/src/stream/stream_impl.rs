use std::sync::{Arc, LazyLock};

use async_nats::{
    jetstream::{
        consumer::AckPolicy,
        kv::{self, CreateErrorKind},
        stream::{self, LastRawMessageErrorKind, State},
    },
    RequestErrorKind,
};
use async_trait::async_trait;
use fuel_streams_macros::subject::IntoSubject;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use tokio::sync::OnceCell;

use crate::prelude::*;

pub static MAX_ACK_PENDING: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("MAX_ACK_PENDING")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(5)
});

#[derive(Debug, Clone)]
pub struct PublishPacket<T: Streamable> {
    pub subject: Arc<dyn IntoSubject>,
    pub payload: Arc<T>,
}

impl<T: Streamable> PublishPacket<T> {
    pub fn new(payload: T, subject: Arc<dyn IntoSubject>) -> Self {
        Self {
            payload: Arc::new(payload),
            subject,
        }
    }

    pub fn get_s3_path(&self) -> String {
        let subject = self.subject.parse();
        format!("{}.json.zstd", subject.replace('.', "/"))
    }
}
pub trait StreamEncoder: DataEncoder<Err = StreamError> {}
impl<T: DataEncoder<Err = StreamError>> StreamEncoder for T {}

/// Trait for types that can be streamed.
///
/// # Examples
///
/// ```no_run
/// use async_trait::async_trait;
/// use fuel_streams_core::prelude::*;
/// use fuel_data_parser::*;
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// struct MyStreamable {
///     data: String,
/// }
///
/// impl DataEncoder for MyStreamable {
///     type Err = StreamError;
/// }
///
/// #[async_trait]
/// impl Streamable for MyStreamable {
///     const NAME: &'static str = "my_streamable";
///     const WILDCARD_LIST: &'static [&'static str] = &["*"];
/// }
/// ```
#[async_trait]
pub trait Streamable: StreamEncoder + std::marker::Sized {
    const NAME: &'static str;
    const WILDCARD_LIST: &'static [&'static str];

    fn to_packet(&self, subject: Arc<dyn IntoSubject>) -> PublishPacket<Self> {
        PublishPacket::new(self.clone(), subject)
    }
}

/// Houses nats-agnostic APIs for publishing and consuming a streamable type
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use fuel_streams_core::prelude::*;
/// use fuel_streams_macros::subject::IntoSubject;
/// use fuel_data_parser::*;
/// use futures::StreamExt;
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// struct MyStreamable {
///     data: String,
/// }
///
/// impl DataEncoder for MyStreamable {
///     type Err = StreamError;
/// }
///
/// #[async_trait::async_trait]
/// impl Streamable for MyStreamable {
///     const NAME: &'static str = "my_streamable";
///     const WILDCARD_LIST: &'static [&'static str] = &["*"];
/// }
///
/// async fn example(nats_client: &NatsClient, storage: &Arc<S3Storage>) {
///     let stream = Stream::<MyStreamable>::new(nats_client, storage).await;
///
///     // Publish
///     let subject = BlocksSubject::new().with_height(Some(23.into())).arc();
///     let packet = MyStreamable { data: "foo".into() }.to_packet(subject);
///     stream.publish(&packet).await.unwrap();
///
///     // Subscribe
///     let mut subscription = stream.subscribe(None).await.unwrap();
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
    store: Arc<kv::Store>,
    storage: Arc<S3Storage>,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Streamable> Stream<S> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(
        nats_client: &NatsClient,
        storage: &Arc<S3Storage>,
    ) -> Self {
        let cell = Self::INSTANCE;
        cell.get_or_init(|| async {
            Self::new(nats_client, storage).await.to_owned()
        })
        .await
        .to_owned()
    }

    pub async fn new(
        nats_client: &NatsClient,
        storage: &Arc<S3Storage>,
    ) -> Self {
        let namespace = &nats_client.namespace;
        let bucket_name = namespace.stream_name(S::NAME);
        let config = kv::Config {
            bucket: bucket_name.to_owned(),
            storage: stream::StorageType::File,
            history: 1,
            ..Default::default()
        };

        let store = nats_client
            .get_or_create_kv_store(config)
            .await
            .expect("Streams must be created");

        Self {
            store: Arc::new(store),
            storage: Arc::clone(storage),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn publish(
        &self,
        packet: &PublishPacket<S>,
    ) -> Result<usize, StreamError> {
        let payload = &packet.payload;
        let s3_path = packet.get_s3_path();
        let subject_name = &packet.subject.parse();
        let encoded = payload.encode().await?;
        self.storage.store(&s3_path, encoded).await?;
        self.publish_s3_path_to_nats(subject_name, &s3_path).await
    }

    async fn publish_s3_path_to_nats(
        &self,
        subject_name: &str,
        s3_path: &str,
    ) -> Result<usize, StreamError> {
        tracing::debug!("S3 path published: {:?}", s3_path);
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

    pub async fn subscribe_raw(
        &self,
        subscription_config: Option<SubscriptionConfig>,
    ) -> Result<BoxStream<'_, (Vec<u8>, NatsMessage)>, StreamError> {
        let config = self.get_consumer_config(subscription_config);
        let config = self.prefix_filter_subjects(config);
        let consumer = self.store.stream.create_consumer(config).await?;
        let messages = consumer.messages().await?;
        let storage = Arc::clone(&self.storage);

        Ok(messages
            .then(move |message| {
                let message = message.expect("Message must be valid");
                let nats_payload = message.payload.to_vec();
                let storage = storage.clone();
                async move {
                    let s3_path = String::from_utf8(nats_payload)
                        .expect("Must be S3 path");
                    (
                        storage
                            .retrieve(&s3_path)
                            .await
                            .expect("S3 object must exist"),
                        message,
                    )
                }
            })
            .boxed())
    }

    pub async fn subscribe(
        &self,
        subscription_config: Option<SubscriptionConfig>,
    ) -> Result<BoxStream<'_, S>, StreamError> {
        let raw_stream = self.subscribe_raw(subscription_config).await?;
        Ok(raw_stream
            .then(|(s3_data, message)| async move {
                let item = S::decode(&s3_data).await.expect("Failed to decode");
                let _ = message.ack().await;
                item
            })
            .boxed())
    }

    pub fn get_consumer_config(
        &self,
        subscription_config: Option<SubscriptionConfig>,
    ) -> PullConsumerConfig {
        let filter_subjects = match subscription_config.clone() {
            Some(subscription_config) => subscription_config.filter_subjects,
            None => vec![S::WILDCARD_LIST[0].to_string()],
        };
        let delivery_policy = match subscription_config.clone() {
            Some(subscription_config) => subscription_config.deliver_policy,
            None => NatsDeliverPolicy::New,
        };
        PullConsumerConfig {
            filter_subjects,
            deliver_policy: delivery_policy,
            ack_policy: AckPolicy::Explicit,
            max_ack_pending: *MAX_ACK_PENDING as i64,
            ..Default::default()
        }
    }

    #[cfg(feature = "test-helpers")]
    /// Fetch all old messages from this stream
    pub async fn catchup(
        &self,
        number_of_messages: usize,
    ) -> Result<BoxStream<'_, Option<S>>, StreamError> {
        let config = PullConsumerConfig {
            filter_subjects: self.all_filter_subjects(),
            deliver_policy: NatsDeliverPolicy::All,
            ack_policy: AckPolicy::None,
            ..Default::default()
        };
        let config = self.prefix_filter_subjects(config);
        let consumer = self.store.stream.create_consumer(config).await?;

        let stream = consumer
            .messages()
            .await?
            .take(number_of_messages)
            .then(|message| async {
                if let Ok(message) = message {
                    Some(
                        self.get_payload_from_s3(message.payload.to_vec())
                            .await
                            .unwrap(),
                    )
                } else {
                    None
                }
            })
            .boxed();

        Ok(stream)
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
            Ok(message) => Ok(Some(
                self.get_payload_from_s3(message.payload.to_vec()).await?,
            )),
            Err(error) => match &error.kind() {
                LastRawMessageErrorKind::NoMessageFound => Ok(None),
                _ => Err(error.into()),
            },
        }
    }

    async fn get_payload_from_s3(
        &self,
        nats_payload: Vec<u8>,
    ) -> Result<S, StreamError> {
        let s3_path = String::from_utf8(nats_payload).expect("Must be S3 path");
        let s3_object = self
            .storage
            .retrieve(&s3_path)
            .await
            .expect("S3 object must exist");
        Ok(S::decode(&s3_object).await.expect("Failed to decode"))
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

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }
}

/// Configuration for subscribing to a consumer.
///
/// # Examples
///
/// ```
/// use fuel_streams_core::stream::SubscriptionConfig;
/// use async_nats::jetstream::consumer::DeliverPolicy as NatsDeliverPolicy;
///
/// let config = SubscriptionConfig {
///     filter_subjects: vec!["example.*".to_string()],
///     deliver_policy: NatsDeliverPolicy::All,
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct SubscriptionConfig {
    pub filter_subjects: Vec<String>,
    pub deliver_policy: NatsDeliverPolicy,
}

#[cfg(any(test, feature = "test-helpers"))]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestStreamable {
        data: String,
    }

    impl DataEncoder for TestStreamable {
        type Err = StreamError;
    }

    #[async_trait]
    impl Streamable for TestStreamable {
        const NAME: &'static str = "test_streamable";
        const WILDCARD_LIST: &'static [&'static str] = &["*"];
    }

    #[tokio::test]
    async fn test_stream_item_s3_encoding_flow() {
        let (stream, _, test_data, subject) = setup_test().await;
        let packet = test_data.to_packet(subject);

        // Publish (this will encode and store in S3)
        stream.publish(&packet).await.unwrap();

        // Get the S3 path that was used
        let s3_path = packet.get_s3_path();

        // Retrieve directly from S3 and verify encoding
        let raw_s3_data = stream.storage.retrieve(&s3_path).await.unwrap();
        let decoded = TestStreamable::decode(&raw_s3_data).await.unwrap();
        assert_eq!(decoded, test_data, "Retrieved data should match original");
    }

    #[tokio::test]
    async fn test_stream_item_json_encoding_flow() {
        use fuel_data_parser::DataParser;
        let (_, _, test_data, _) = setup_test().await;
        let encoded = test_data.encode().await.unwrap();
        let decoded = TestStreamable::decode(&encoded).await.unwrap();
        assert_eq!(decoded, test_data, "Decoded data should match original");

        let json = DataParser::default().encode_json(&test_data).unwrap();
        let json_str = String::from_utf8(json).unwrap();
        let expected_json = r#"{"data":"test content"}"#;
        assert_eq!(
            json_str, expected_json,
            "JSON structure should exactly match expected format"
        );
    }

    #[cfg(test)]
    async fn setup_test() -> (
        Stream<TestStreamable>,
        Arc<S3Storage>,
        TestStreamable,
        Arc<dyn IntoSubject>,
    ) {
        let storage = S3Storage::new_for_testing().await.unwrap();
        let nats_client_opts =
            NatsClientOpts::admin_opts().with_rdn_namespace();
        let nats_client = NatsClient::connect(&nats_client_opts).await.unwrap();
        let stream = Stream::<TestStreamable>::new(
            &nats_client,
            &Arc::new(storage.clone()),
        )
        .await;
        let test_data = TestStreamable {
            data: "test content".to_string(),
        };
        let subject = Arc::new(
            BlocksSubject::new()
                .with_producer(Some(Address::zeroed()))
                .with_height(Some(1.into())),
        );
        (stream, Arc::new(storage), test_data, subject)
    }
}

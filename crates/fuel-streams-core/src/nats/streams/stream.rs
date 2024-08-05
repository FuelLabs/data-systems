use std::fmt::Debug;

use futures_util::stream::Take;

use super::Subject;
use crate::{
    nats::{types::*, NatsClient, NatsError},
    prelude::NatsNamespace,
    types::BoxedResult,
};

pub trait Streamable: Debug + Clone {
    const STREAM: &'static str;
    const SUBJECTS_WILDCARDS: &'static [&'static str];
}

#[derive(Clone)]
pub struct Stream<S: Streamable> {
    pub stream: AsyncNatsStream,
    pub(self) namespace: NatsNamespace,
    _marker: std::marker::PhantomData<S>,
}

impl<S: Streamable> Stream<S> {
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let subjects = client
            .namespace
            .prepend_subjects(S::SUBJECTS_WILDCARDS.to_vec());
        let config = JetStreamConfig {
            subjects,
            storage: NatsStorageType::File,
            ..Default::default()
        };

        let stream = client.create_stream(S::STREAM, config).await?;

        Ok(Stream {
            stream,
            namespace: client.namespace.to_owned(),
            _marker: std::marker::PhantomData,
        })
    }

    pub async fn create_pull_consumer(
        &self,
        client: &NatsClient,
        config: Option<PullConsumerConfig>,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        client
            .create_pull_consumer(S::STREAM, &self.stream, config)
            .await
    }
}

#[cfg(feature = "test-helpers")]
impl<S: Streamable> Stream<S> {
    pub async fn assert_consumer_name(
        &self,
        client: &NatsClient,
        mut consumer: NatsConsumer<PullConsumerConfig>,
    ) -> BoxedResult<()> {
        use pretty_assertions::assert_eq;
        // Checking consumer name created with consumer_from method
        let consumer_info = consumer.info().await.unwrap();
        let consumer_name = consumer_info.clone().config.durable_name.unwrap();
        assert_eq!(consumer_name, client.namespace.consumer_name(S::STREAM));
        Ok(())
    }

    pub async fn assert_messages_consumed(
        &self,
        mut messages: Take<PullConsumerStream>,
        subject: impl Subject,
        payload_data: &'static str,
    ) -> BoxedResult<Take<PullConsumerStream>> {
        use std::str::from_utf8;

        use futures_util::StreamExt;
        use pretty_assertions::assert_eq;

        let parsed = subject.parse();
        if let Some(message) = messages.next().await {
            let message = message?;
            let payload = from_utf8(&message.payload);
            let subject_name = self.namespace.subject_name(&parsed);
            assert_eq!(message.subject.as_str(), subject_name);
            assert_eq!(payload.unwrap(), payload_data.to_string());
            message.ack().await.unwrap();
        }

        Ok(messages)
    }
}

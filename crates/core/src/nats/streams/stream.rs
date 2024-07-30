use std::fmt::{Debug, Display};

use futures_util::stream::Take;
use strum::IntoEnumIterator;

use super::Subject;
use crate::{
    nats::{types::*, NatsClient, NatsError},
    types::BoxedResult,
};

pub trait StreamIdentifier {
    const STREAM: &'static str;

    fn name() -> &'static str {
        Self::STREAM
    }
}

pub trait StreamSubjects: Display + Debug + Clone + IntoEnumIterator {
    fn wildcards(prefix: &str) -> Vec<String> {
        Self::iter().map(|s| format!("{prefix}.{s}")).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Stream<S: StreamSubjects> {
    pub stream: AsyncNatsStream,
    pub prefix: String,
    _marker: std::marker::PhantomData<S>,
}

impl<S: StreamSubjects> Stream<S>
where
    Self: StreamIdentifier,
{
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let prefix = client.conn_id.clone();
        let subjects = S::wildcards(&prefix);
        let stream =
            create_stream(client, Self::STREAM, subjects.to_owned()).await?;

        Ok(Stream {
            stream,
            prefix,
            _marker: std::marker::PhantomData,
        })
    }

    pub async fn create_pull_consumer(
        &self,
        client: &NatsClient,
        config: Option<PullConsumerConfig>,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        client
            .create_pull_consumer(Self::STREAM, &self.stream, config)
            .await
    }
}

async fn create_stream(
    client: &NatsClient,
    name: &str,
    subjects: Vec<String>,
) -> Result<AsyncNatsStream, NatsError> {
    let config = JetStreamConfig {
        subjects,
        storage: NatsStorageType::File,
        ..Default::default()
    };

    client.create_stream(name, config).await
}

#[cfg(any(test, feature = "test_helpers"))]
impl<S: StreamSubjects> Stream<S>
where
    Self: StreamIdentifier,
{
    pub async fn assert_consumer_name(
        &self,
        client: &NatsClient,
        mut consumer: NatsConsumer<PullConsumerConfig>,
    ) -> BoxedResult<()> {
        // Checking consumer name created with consumer_from method
        let consumer_info = consumer.info().await.unwrap();
        let consumer_name = consumer_info.clone().config.durable_name.unwrap();
        assert_eq!(consumer_name, client.consumer_name(Self::STREAM));
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
            let subject_prefixed = format!("{}.{parsed}", self.prefix);
            assert_eq!(message.subject.as_str(), subject_prefixed);
            assert_eq!(payload.unwrap(), payload_data.to_string());
            message.ack().await.unwrap();
        }

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use super::*;

    #[derive(Debug, Clone, strum::EnumIter)]
    enum TestSubjects {
        Test1,
        Test2,
    }

    impl fmt::Display for TestSubjects {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                TestSubjects::Test1 => write!(f, "test1"),
                TestSubjects::Test2 => write!(f, "test2"),
            }
        }
    }

    impl StreamSubjects for TestSubjects {}
    impl StreamIdentifier for Stream<TestSubjects> {
        const STREAM: &'static str = "test_stream";
    }

    #[test]
    fn subjects_wildcards() {
        let wildcards = TestSubjects::wildcards("prefix");
        assert_eq!(wildcards, vec!["prefix.test1", "prefix.test2"]);
    }

    #[test]
    fn identifier() {
        assert_eq!(Stream::<TestSubjects>::STREAM, "test_stream");
    }

    #[test]
    fn subjects_display() {
        assert_eq!(TestSubjects::Test1.to_string(), "test1");
        assert_eq!(TestSubjects::Test2.to_string(), "test2");
    }

    #[test]
    fn subjects_iteration() {
        let subjects: Vec<TestSubjects> = TestSubjects::iter().collect();
        assert_eq!(subjects.len(), 2);
        assert!(matches!(subjects[0], TestSubjects::Test1));
        assert!(matches!(subjects[1], TestSubjects::Test2));
    }
}

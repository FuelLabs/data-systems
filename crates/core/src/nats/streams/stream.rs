use std::fmt::{Debug, Display};

use strum::IntoEnumIterator;

use crate::nats::{types, NatsClient, NatsError};

pub trait StreamIdentifier {
    const STREAM: &'static str;
}

pub trait StreamSubjectsEnum:
    Display + Debug + Clone + IntoEnumIterator
{
    fn wildcards(prefix: &str) -> Vec<String> {
        Self::iter().map(|s| format!("{prefix}.{s}")).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Stream<S: StreamSubjectsEnum> {
    stream: types::AsyncNatsStream,
    _marker: std::marker::PhantomData<S>,
}

impl<S: StreamSubjectsEnum> Stream<S>
where
    Self: StreamIdentifier,
{
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let subjects = S::wildcards(client.conn_id.as_str());
        let stream = create_stream(client, Self::STREAM, subjects).await?;
        Ok(Stream {
            stream,
            _marker: std::marker::PhantomData,
        })
    }
    pub fn stream(&self) -> &types::AsyncNatsStream {
        &self.stream
    }
}

async fn create_stream(
    client: &NatsClient,
    name: &str,
    subjects: Vec<String>,
) -> Result<types::AsyncNatsStream, NatsError> {
    let config = types::JetStreamConfig {
        subjects,
        storage: types::NatsStorageType::File,
        ..Default::default()
    };

    client.create_stream(name, config).await
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

    impl StreamSubjectsEnum for TestSubjects {}
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

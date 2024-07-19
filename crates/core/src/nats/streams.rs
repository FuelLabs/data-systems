use std::collections::HashMap;

use futures_util::future::try_join_all;

use super::client::NatsClient;
use super::types::{
    JetStreamConfig,
    JetStreamContext,
    NatsStorageType,
    NatsStream,
};
use super::{NatsError, SubjectName, Subjects};

pub type StreamMap = HashMap<SubjectName, NatsStream>;

#[derive(Debug, Clone)]
pub struct Streams {
    pub stream_map: StreamMap,
    pub subjects: Subjects,
}

impl Streams {
    pub async fn new(
        client: &NatsClient,
        jetstream: &JetStreamContext,
    ) -> Result<Self, NatsError> {
        let subjects = Subjects::new(&client.conn_id);
        let stream_futures = subjects.to_owned().map.into_iter().map(
            |(key, value)| async move {
                let entity = key.entity();
                let stream_subjects = value;
                let config = JetStreamConfig {
                    subjects: stream_subjects,
                    storage: NatsStorageType::File,
                    ..Default::default()
                };
                let stream =
                    client.create_stream(jetstream, entity, config).await?;
                Ok((key, stream))
            },
        );

        let stream_map: StreamMap =
            try_join_all(stream_futures).await?.into_iter().collect();

        Ok(Self {
            stream_map,
            subjects,
        })
    }

    pub fn stream_of(&self, name: &SubjectName) -> Option<&NatsStream> {
        self.stream_map.get(name)
    }

    pub fn subjects_of(&self, name: &SubjectName) -> Option<&Vec<String>> {
        self.subjects.map.get(name)
    }
}

#[cfg(test)]
mod tests {
    use async_nats::jetstream::stream::Stream;
    use mockall::mock;

    use super::*;

    mock! {
        NatsClient {}
        impl NatsClient {
            pub async fn create_stream(
                &self,
                context: &JetStreamContext,
                name: &str,
                config: JetStreamConfig,
            ) -> Result<NatsStream, NatsError>;
        }
    }

    mock! {
        JetStreamContext {}
    }

    #[tokio::test]
    async fn test_streams_new() {
        let mut mock_client = MockNatsClient::new();
        let mock_context = MockJetStreamContext::new();

        mock_client.expect_create_stream()
            .times(2)  // We expect it to be called twice, once for each SubjectName
            .returning(|_, name, _| {
                Ok(Stream::new())
            });

        let client = mock_client;
        let jetstream = &mock_context;

        let streams = Streams::new(&client, jetstream).await.unwrap();

        assert_eq!(streams.stream_map.len(), 2);
        assert!(streams.stream_map.contains_key(&SubjectName::Blocks));
        assert!(streams.stream_map.contains_key(&SubjectName::Transactions));
    }

    #[test]
    fn test_stream_of() {
        let mut stream_map = StreamMap::new();
        stream_map.insert(SubjectName::Blocks, Stream::new());

        let subjects = Subjects::new("test_prefix");
        let streams = Streams {
            stream_map,
            subjects,
        };

        assert!(streams.stream_of(&SubjectName::Blocks).is_some());
        assert!(streams.stream_of(&SubjectName::Transactions).is_none());
    }

    #[test]
    fn test_subjects_of() {
        let stream_map = StreamMap::new();
        let subjects = Subjects::new("test_prefix");
        let streams = Streams {
            stream_map,
            subjects,
        };

        let blocks_subjects =
            streams.subjects_of(&SubjectName::Blocks).unwrap();
        assert_eq!(blocks_subjects, &vec!["test_prefix.blocks.*".to_string()]);

        let transactions_subjects =
            streams.subjects_of(&SubjectName::Transactions).unwrap();
        assert_eq!(
            transactions_subjects,
            &vec!["test_prefix.transactions.*.*.*".to_string()]
        );
    }
}

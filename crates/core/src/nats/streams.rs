use std::collections::HashMap;

use futures_util::future::try_join_all;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::{
    client::NatsClient,
    types::{
        JetStreamConfig,
        NatsConsumer,
        NatsStorageType,
        NatsStream,
        PullConsumerConfig,
    },
    NatsError,
    SubjectName,
};

#[derive(Debug, EnumIter, Clone, Hash, Eq, PartialEq)]
pub enum StreamKind {
    Blocks,
    Transactions,
}

impl StreamKind {
    pub fn get_subjects(&self, prefix: &str) -> Vec<String> {
        match self {
            Self::Blocks => vec![SubjectName::Blocks.with_prefix(prefix)],
            Self::Transactions => vec![
                SubjectName::Transactions.with_prefix(prefix),
                SubjectName::TransactionsById.with_prefix(prefix),
            ],
        }
    }

    pub fn get_storage(&self) -> NatsStorageType {
        match self {
            Self::Blocks => NatsStorageType::File,
            Self::Transactions => NatsStorageType::File,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Blocks => "blocks",
            Self::Transactions => "transactions",
        }
    }

    pub fn get_stream_config(&self, prefix: &str) -> JetStreamConfig {
        JetStreamConfig {
            subjects: self.get_subjects(prefix),
            storage: self.get_storage(),
            ..Default::default()
        }
    }
}

pub type StreamMap = HashMap<StreamKind, NatsStream>;

#[derive(Debug, Clone)]
pub struct Streams {
    pub client: NatsClient,
    pub prefix: String,
    pub map: StreamMap,
}

impl Streams {
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let prefix = client.conn_id.as_str();
        let stream_futures = StreamKind::iter().map(|stream_kind| async move {
            let name = stream_kind.get_name();
            let config = stream_kind.get_stream_config(prefix);
            let nats_stream = client.create_stream(name, config).await?;
            Ok((stream_kind, nats_stream))
        });

        let stream_map: StreamMap =
            try_join_all(stream_futures).await?.into_iter().collect();

        Ok(Self {
            client: client.clone(),
            prefix: prefix.to_string(),
            map: stream_map,
        })
    }

    pub fn stream_of(&self, kind: &StreamKind) -> Option<&NatsStream> {
        self.map.get(kind)
    }

    pub async fn consumer_from_stream(
        &self,
        kind: &StreamKind,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        let name = &kind.get_name();
        let stream = self.stream_of(kind).unwrap();
        self.client.create_pull_consumer(name, stream, None).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use futures_util::StreamExt;

    use super::*;
    use crate::{
        nats::{client::NatsClient, Subject},
        types::BoxedResult,
    };

    #[tokio::test]
    async fn new_instance() -> BoxedResult<()> {
        let (client, cleanup) =
            NatsClient::connect_with_testcontainer().await?;
        let streams = Streams::new(&client).await?;

        assert_eq!(streams.prefix, client.conn_id);
        assert_eq!(streams.map.len(), StreamKind::iter().count());

        cleanup.await?;
        Ok(())
    }

    #[tokio::test]
    async fn testing_stream_map() -> BoxedResult<()> {
        let (client, cleanup) =
            NatsClient::connect_with_testcontainer().await?;

        let streams = Streams::new(&client).await?;
        for kind in StreamKind::iter() {
            assert!(
                streams.stream_of(&kind).is_some(),
                "Stream {:?} not found",
                kind
            );
        }

        cleanup.await?;
        Ok(())
    }

    #[tokio::test]
    async fn consume_stream() -> BoxedResult<()> {
        let (client, cleanup) =
            NatsClient::connect_with_testcontainer().await?;
        let jetstream = client.jetstream.as_ref();
        let streams = Streams::new(&client).await?;
        let mut consumer =
            streams.consumer_from_stream(&StreamKind::Blocks).await?;

        // Check if the consumer was created with the correct name
        let info = consumer.info().await?;
        let name = info.config.durable_name.clone().unwrap();
        assert_eq!(name, "test__consumer:blocks");

        // Publish 10 messages to the blocks stream
        for i in 0..10 {
            let subject = Subject::Blocks {
                producer: format!("0x00{}", i),
                height: i,
            };
            jetstream
                .publish(subject.with_prefix(&streams.prefix), "data".into())
                .await?;
        }

        // Consume the messages and check if they are correct
        let mut messages = consumer.messages().await?.take(10);
        let mut count = 0;
        while let Some(message) = messages.next().await {
            let message = message?;
            let payload = from_utf8(&message.payload);
            assert_eq!(
                message.subject.as_str(),
                format!("test.blocks.0x00{count}.{count}")
            );
            assert_eq!(payload.unwrap(), "data");
            message.ack().await.unwrap();
            count += 1;
        }

        cleanup.await?;
        Ok(())
    }
}

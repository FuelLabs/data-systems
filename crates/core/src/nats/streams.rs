use std::collections::HashMap;

use futures_util::future::try_join_all;
use futures_util::StreamExt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::client::NatsClient;
use super::types::{
    JetStreamConfig,
    JetStreamContext,
    NatsMessage,
    NatsStorageType,
    NatsStream,
};
use super::{NatsError, SubjectName};
use crate::types::BoxedResult;

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
    client: NatsClient,
    pub prefix: String,
    pub map: StreamMap,
}

impl Streams {
    pub async fn new(
        client: &NatsClient,
        jetstream: &JetStreamContext,
    ) -> Result<Self, NatsError> {
        let prefix = client.conn_id.as_str();
        let stream_futures = StreamKind::iter().map(|stream_kind| async move {
            let name = stream_kind.get_name();
            let config = stream_kind.get_stream_config(prefix);
            let nats_stream =
                client.create_stream(jetstream, name, config).await?;
            Ok((stream_kind, nats_stream))
        });

        let stream_map: StreamMap =
            try_join_all(stream_futures).await?.into_iter().collect();

        Ok(Self {
            client: client.to_owned(),
            prefix: prefix.to_string(),
            map: stream_map,
        })
    }

    pub fn stream_of(&self, kind: &StreamKind) -> Option<&NatsStream> {
        self.map.get(kind)
    }

    pub async fn consume_stream(
        &self,
        kind: &StreamKind,
        handler: impl Fn(NatsMessage) -> Result<(), NatsError>
            + Send
            + Sync
            + 'static,
    ) -> BoxedResult<()> {
        let name = &kind.get_name();
        let stream = self.stream_of(kind);
        let consumer =
            self.client.create_pull_consumer(name, stream, None).await?;

        let mut messages = consumer.messages().await?;
        while let Some(message) = messages.next().await {
            let message = message?;
            handler(message)?;
        }

        Ok(())
    }
}

use async_nats::jetstream::consumer::PullConsumer;
use bytes::Bytes;
use futures_util::StreamExt;

use super::types::{JetStreamContext, NatsMessage, NatsStream};
use super::{NatsClient, NatsError, Streams, Subject, SubjectName};
use crate::types::BoxedResult;

#[derive(Debug, Clone)]
pub struct Nats {
    pub conn_id: String,
    pub client: NatsClient,
    pub jetstream: JetStreamContext,
    pub streams: Streams,
}

impl Nats {
    pub async fn new(
        conn_id: String,
        nats_url: &str,
        nats_nkey: Option<String>,
    ) -> Result<Self, NatsError> {
        let client = NatsClient::connect(nats_url, nats_nkey, &conn_id).await?;
        let jetstream = async_nats::jetstream::new(client.to_owned().into());
        let streams = Streams::new(&client, &jetstream).await?;

        Ok(Self {
            conn_id,
            client,
            jetstream,
            streams,
        })
    }

    pub async fn publish(
        &self,
        subject: Subject,
        payload: Bytes,
    ) -> BoxedResult<()> {
        let subject = subject.with_prefix(&self.conn_id);

        let ack_future = self.jetstream.publish(subject, payload).await?;
        ack_future.await?;
        Ok(())
    }

    pub async fn consume(
        &self,
        consumer: PullConsumer,
        handler: impl Fn(NatsMessage) -> Result<(), NatsError>
            + Send
            + Sync
            + 'static,
    ) -> BoxedResult<()> {
        let mut messages = consumer.messages().await?;
        while let Some(message) = messages.next().await {
            let message = message?;
            handler(message)?;
        }

        Ok(())
    }

    pub fn get_client(&self) -> &NatsClient {
        &self.client
    }

    pub fn get_stream(&self, subject: &SubjectName) -> Option<&NatsStream> {
        self.streams.stream_of(subject)
    }

    pub fn get_subjects(&self, subject: &SubjectName) -> Option<&Vec<String>> {
        self.streams.subjects_of(subject)
    }
}

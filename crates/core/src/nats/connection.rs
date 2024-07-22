use bytes::Bytes;

use super::types::JetStreamContext;
use super::{NatsClient, NatsError, Streams, Subject};
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
}

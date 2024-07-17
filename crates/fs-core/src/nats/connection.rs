use async_nats::connection;
use futures_util::StreamExt;
use tracing::info;

use super::types::{
    JetStreamConfig,
    JetStreamContext,
    NatsConnectOptions,
    NatsConsumer,
    NatsMessage,
    NatsStorageType,
    NatsStream,
    PullConsumerConfig,
};
use super::{NatsError, Subject, Subjects};
use crate::types::{Bytes, PayloadSize, Result};

#[derive(Debug, Clone)]
pub struct NatsClient {
    pub url: String,
    pub conn_id: String,
    pub client: async_nats::Client,
    max_payload_size: PayloadSize,
}

impl NatsClient {
    pub async fn connect(
        url: impl ToString,
        nkey: Option<String>,
        conn_id: &impl ToString,
    ) -> Result<Self> {
        let conn_id = conn_id.to_string();
        let url = url.to_string();
        let options =
            nkey.map(NatsConnectOptions::with_nkey).unwrap_or_default();

        let client = async_nats::connect_with_options(&url, options)
            .await
            .map_err(|e| NatsError::ConnectError {
                url: url.to_owned(),
                source: e,
            })?;

        let max_payload_size = client.server_info().max_payload;
        info!("NATS Connected: url={url} max_payload_size={max_payload_size}");

        Ok(Self {
            url,
            conn_id,
            client,
            max_payload_size,
        })
    }

    pub fn is_connected(&self) -> Result<&Self> {
        let client = self.client.to_owned();
        let conn_state = client.connection_state();
        match conn_state {
            connection::State::Pending => {
                anyhow::bail!(NatsError::ConnectionPending(self.url.to_owned()))
            }
            connection::State::Disconnected => {
                anyhow::bail!(NatsError::ConnectionDisconnected(
                    self.url.to_owned()
                ))
            }
            connection::State::Connected => Ok(self),
        }
    }

    pub fn validate_payload(
        &self,
        payload: Bytes,
        subject: Subject,
    ) -> Result<()> {
        let payload_size = payload.len();
        let max_payload_size = self.max_payload_size;
        if payload_size > max_payload_size {
            anyhow::bail!(NatsError::PayloadTooLarge {
                subject,
                payload_size,
                max_payload_size
            })
        }

        Ok(())
    }

    pub async fn create_stream(
        &self,
        context: &JetStreamContext,
        name: &str,
        subjects: Subjects,
    ) -> Result<NatsStream> {
        let stream = context
            .get_or_create_stream(JetStreamConfig {
                name: format!("{}_{}", self.conn_id, name),
                storage: NatsStorageType::File,
                subjects: subjects.into(),
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::CreateStreamFailed { source: e })?;

        Ok(stream)
    }

    pub async fn create_pull_consumer(
        &self,
        stream: &NatsStream,
        name: &str,
        subjects: Subjects,
    ) -> Result<NatsConsumer<PullConsumerConfig>> {
        let name = format!("{}_consumer_{}", self.conn_id, name);
        let consumer = stream
            .get_or_create_consumer(
                name.as_str(),
                PullConsumerConfig {
                    durable_name: Some(name.to_owned()),
                    filter_subjects: subjects.into(),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| NatsError::CreateConsumerFailed { source: e })?;

        Ok(consumer)
    }
}

impl From<NatsClient> for async_nats::Client {
    fn from(client: NatsClient) -> Self {
        client.client.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Nats {
    pub conn_id: String,
    pub client: NatsClient,
    pub context: JetStreamContext,
    pub main_stream: NatsStream,
}

impl Nats {
    pub async fn new(
        conn_id: String,
        nats_url: &str,
        nats_nkey: Option<String>,
    ) -> Result<Self> {
        let client = NatsClient::connect(nats_url, nats_nkey, &conn_id).await?;
        let context = async_nats::jetstream::new(client.to_owned().into());
        let subjects = Subjects::build_all(&conn_id);
        let main_stream =
            client.create_stream(&context, "fuel", subjects).await?;

        Ok(Self {
            conn_id,
            client,
            context,
            main_stream,
        })
    }

    pub async fn publish(
        &self,
        subject: Subject,
        payload: Bytes,
    ) -> Result<()> {
        let _conn = self.client.is_connected()?;
        let subject = subject.with_prefix(&self.conn_id);
        let ack_future = self.context.publish(subject, payload).await?;
        ack_future.await?;
        Ok(())
    }

    pub async fn consume(
        &self,
        name: &impl ToString,
        subjects: Subjects,
        handler: impl Fn(NatsMessage) -> Result<()> + Send + Sync + 'static,
    ) -> Result<()> {
        let client = self.client.to_owned();
        let _conn = client.is_connected()?;
        let consumer = client
            .create_pull_consumer(
                &self.main_stream,
                name.to_string().as_str(),
                subjects,
            )
            .await?;

        let mut messages = consumer.messages().await?;
        while let Some(message) = messages.next().await {
            let message = message?;
            handler(message)?;
        }

        Ok(())
    }

    pub fn client(&self) -> &NatsClient {
        &self.client
    }
}

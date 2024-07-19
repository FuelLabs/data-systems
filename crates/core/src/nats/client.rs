use async_nats::jetstream::consumer::{FromConsumer, IntoConsumerConfig};
use bytes::Bytes;
use tracing::info;

use super::types::{
    JetStreamConfig,
    JetStreamContext,
    NatsConnectOptions,
    NatsConsumer,
    NatsStream,
    PayloadSize,
};
use super::{NatsError, Subject};

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
    ) -> Result<Self, NatsError> {
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

    pub fn validate_payload(
        &self,
        payload: Bytes,
        subject: Subject,
    ) -> Result<(), NatsError> {
        let payload_size = payload.len();
        let max_payload_size = self.max_payload_size;
        if payload_size > max_payload_size {
            return Err(NatsError::PayloadTooLarge {
                subject,
                payload_size,
                max_payload_size,
            });
        }

        Ok(())
    }

    pub async fn create_stream(
        &self,
        context: &JetStreamContext,
        name: &str,
        config: JetStreamConfig,
    ) -> Result<NatsStream, NatsError> {
        let name = format!("{}_{}", self.conn_id, name);
        let stream = context
            .get_or_create_stream(JetStreamConfig {
                name: name.to_owned(),
                ..config
            })
            .await
            .map_err(|e| NatsError::CreateStreamFailed { name, source: e })?;

        Ok(stream)
    }

    pub async fn create_consumer<C: FromConsumer + IntoConsumerConfig>(
        &self,
        name: &str,
        stream: &NatsStream,
        config: C,
    ) -> Result<NatsConsumer<C>, NatsError> {
        let name = format!("{}_consumer_{}", self.conn_id, name);
        let consumer = stream
            .get_or_create_consumer(name.as_str(), config)
            .await
            .map_err(|e| NatsError::CreateConsumerFailed {
                name: name.to_owned(),
                source: e,
            })?;

        Ok(consumer)
    }
}

impl From<NatsClient> for async_nats::Client {
    fn from(client: NatsClient) -> Self {
        client.client.clone()
    }
}

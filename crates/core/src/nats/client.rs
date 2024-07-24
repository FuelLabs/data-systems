use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use tracing::info;

use super::{
    types::{
        JetStreamConfig,
        JetStreamContext,
        NatsConsumer,
        NatsStream,
        PullConsumerConfig,
    },
    NatsError,
    Subject,
};
use crate::types::BoxedResult;

#[derive(Debug, Clone)]
pub struct NatsClient {
    pub url: String,
    pub conn_id: String,
    pub conn: Arc<async_nats::Client>,
    pub jetstream: Arc<JetStreamContext>,
}

impl NatsClient {
    pub async fn connect(
        url: &str,
        conn_id: &str,
        nkey: Option<String>,
    ) -> Result<Self, NatsError> {
        let conn_id = conn_id.to_string();
        let conn = Self::create_conn(url, nkey).await?;
        let context =
            Arc::new(async_nats::jetstream::new(conn.as_ref().clone()));

        info!("Connected to NATS server at {}", url);
        Ok(Self {
            url: url.to_string(),
            conn_id,
            conn,
            jetstream: context,
        })
    }

    async fn create_conn(
        url: &str,
        nkey: Option<String>,
    ) -> Result<Arc<async_nats::Client>, NatsError> {
        let options = async_nats::ConnectOptions::new()
            .connection_timeout(Duration::from_secs(30))
            .max_reconnects(10);

        if nkey.is_none() {
            return Err(NatsError::NkeyNotProvided {
                url: url.to_owned(),
            })
        }

        let conn =
            async_nats::connect_with_options(&url, options.nkey(nkey.unwrap()))
                .await
                .map_err(|e| NatsError::ConnectError {
                    url: url.to_owned(),
                    source: e,
                })?;

        Ok(Arc::new(conn))
    }

    pub fn validate_payload(
        &self,
        payload: &Bytes,
        subject_name: &str,
    ) -> Result<&Self, NatsError> {
        let payload_size = payload.len();
        let max_payload_size = self.conn.server_info().max_payload;
        if payload_size > max_payload_size {
            return Err(NatsError::PayloadTooLarge {
                subject: subject_name.to_string(),
                payload_size,
                max_payload_size,
            });
        }

        Ok(self)
    }

    pub async fn publish(
        &self,
        subject: Subject,
        payload: Bytes,
    ) -> BoxedResult<&Self> {
        let subject_prefixed = subject.with_prefix(&self.conn_id);
        let context = self.jetstream.as_ref();
        let ack_future = context.publish(subject_prefixed, payload).await?;
        ack_future.await?;
        Ok(self)
    }

    pub(crate) fn stream_name(&self, val: &str) -> String {
        let id = self.conn_id.clone();
        format!("{id}_stream:{val}")
    }

    pub(crate) fn consumer_name(&self, val: &str) -> String {
        let id = self.conn_id.clone();
        format!("{id}_consumer:{val}")
    }

    pub async fn create_stream(
        &self,
        name: &str,
        config: JetStreamConfig,
    ) -> Result<NatsStream, NatsError> {
        let name = self.stream_name(name);
        let context = self.jetstream.as_ref();
        context
            .create_stream(JetStreamConfig {
                name: name.clone(),
                ..config
            })
            .await
            .map_err(|e| NatsError::CreateStreamFailed {
                name: name.clone(),
                source: e,
            })
    }

    pub async fn create_pull_consumer(
        &self,
        name: &str,
        stream: &NatsStream,
        config: Option<PullConsumerConfig>,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        let name = self.consumer_name(name);
        stream
            .get_or_create_consumer(
                name.as_str(),
                PullConsumerConfig {
                    durable_name: Some(name.to_owned()),
                    ..config.unwrap_or_default()
                },
            )
            .await
            .map_err(|e| NatsError::CreateConsumerFailed {
                name: name.clone(),
                source: e,
            })
    }
}

/// Tests helpers
impl NatsClient {
    #[cfg(test)]
    pub async fn connect_when_testing(
        connection_id: Option<String>,
    ) -> Result<NatsClient, NatsError> {
        let host = "nats://localhost:4222".to_string();
        let url = &dotenvy::var("NATS_URL").unwrap_or(host);
        let nkey = dotenvy::var("NATS_NKEY_SEED").unwrap();
        let conn_id = if let Some(id) = connection_id {
            id
        } else {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let random_int: u32 = rng.gen();
            format!(r"connection-{random_int}")
        };

        NatsClient::connect(url, conn_id.as_str(), Some(nkey)).await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn validate_payload_size() -> BoxedResult<()> {
        let client = NatsClient::connect_when_testing(None).await?;

        // Test with a payload within the size limit
        let small_payload = Bytes::from(vec![0; 100]);
        assert!(client
            .validate_payload(&small_payload, "test.subject")
            .is_ok());

        // Test with a payload exceeding the size limit
        let max_payload_size = client.conn.server_info().max_payload;
        let large_payload = Bytes::from(vec![0; max_payload_size + 1]);
        assert!(client
            .validate_payload(&large_payload, "test.subject")
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn create_stream_and_consumer() -> BoxedResult<()> {
        let client = NatsClient::connect_when_testing(None).await?;
        let mut stream = client
            .create_stream("test_stream", JetStreamConfig::default())
            .await?;

        let stream_info = stream.info().await?;
        let name = stream_info.config.name.clone();
        assert_eq!(name, client.stream_name("test_stream"));

        let mut consumer = client
            .create_pull_consumer("test_consumer", &stream, None)
            .await?;

        let consumer_info = consumer.info().await?;
        let name = consumer_info.config.durable_name.clone().unwrap();
        assert_eq!(name, client.consumer_name("test_consumer"));

        Ok(())
    }
}

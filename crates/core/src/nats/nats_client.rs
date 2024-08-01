use std::time::Duration;

use async_nats::{
    jetstream::context::{Publish, Streams},
    ConnectOptions,
};
use bytes::Bytes;
use tracing::info;

use super::{
    types::{
        AsyncNatsStream,
        JetStreamConfig,
        JetStreamContext,
        NatsConsumer,
        PullConsumerConfig,
    },
    ConnId,
    NatsError,
};
use crate::types::BoxedResult;

#[derive(Debug, Clone, Default)]
pub struct NatsClient {
    pub url: String,
    pub conn_id: ConnId,
    pub conn: Option<async_nats::Client>,
    pub jetstream: Option<JetStreamContext>,
}

impl NatsClient {
    pub fn new(url: impl ToString, conn_id: ConnId) -> Self {
        Self {
            url: url.to_string(),
            conn_id,
            ..Default::default()
        }
    }

    pub async fn connect(
        self,
        auth_user: &str,
        auth_pass: &str,
    ) -> Result<Self, NatsError> {
        let conn = self.create_conn(auth_pass, auth_user).await?;
        let context = async_nats::jetstream::new(conn.to_owned());

        info!("Connected to NATS server at {}", &self.url);
        Ok(Self {
            url: self.url,
            conn_id: self.conn_id,
            conn: Some(conn),
            jetstream: Some(context),
        })
    }

    async fn create_conn(
        &self,
        auth_pass: &str,
        auth_user: &str,
    ) -> Result<async_nats::Client, NatsError> {
        let options = ConnectOptions::with_user_and_password(
            auth_user.to_owned(),
            auth_pass.to_owned(),
        )
        .connection_timeout(Duration::from_secs(5))
        .name(&self.conn_id)
        .max_reconnects(1);

        async_nats::connect_with_options(&self.url, options)
            .await
            .map_err(|e| NatsError::ConnectionError {
                url: self.url.to_owned(),
                source: e,
            })
    }

    pub fn validate_payload(
        &self,
        payload: &Bytes,
        subject_name: &str,
    ) -> Result<&Self, NatsError> {
        let payload_size = payload.len();
        let conn = self.conn.clone().unwrap();
        let max_payload_size = conn.server_info().max_payload;
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
        subject: String,
        payload: Bytes,
    ) -> BoxedResult<&Self> {
        let subject_prefixed = format!("{}.{subject}", self.conn_id);
        let publish_payload =
            Publish::build().message_id(subject).payload(payload);

        self.jetstream
            .clone()
            .unwrap()
            .send_publish(subject_prefixed, publish_payload)
            .await?
            .await?;

        Ok(self)
    }

    pub fn stream_name(&self, val: &str) -> String {
        format!("{}:stream:{val}", self.conn_id)
    }

    pub fn consumer_name(&self, val: &str) -> String {
        format!("{}:consumer:{val}", self.conn_id)
    }

    pub async fn create_stream(
        &self,
        name: &str,
        config: JetStreamConfig,
    ) -> Result<AsyncNatsStream, NatsError> {
        let name = self.stream_name(name);
        self.jetstream
            .clone()
            .unwrap()
            .get_or_create_stream(JetStreamConfig {
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
        stream: &AsyncNatsStream,
        config: Option<PullConsumerConfig>,
    ) -> Result<NatsConsumer<PullConsumerConfig>, NatsError> {
        let name = self.consumer_name(name);
        stream
            .get_or_create_consumer(
                &name,
                PullConsumerConfig {
                    durable_name: Some(name.to_owned()),
                    ..config.unwrap_or_default()
                },
            )
            .await
            .map_err(|e| NatsError::CreateConsumerFailed { source: e })
    }

    pub fn context_streams(&self) -> Streams {
        self.jetstream.clone().unwrap().streams()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    async fn connect() -> Result<NatsClient, NatsError> {
        let host = "nats://localhost:4222".to_string();
        let pass = dotenvy::var("NATS_ADMIN_PASS").unwrap();
        let url = dotenvy::var("NATS_URL").unwrap_or(host);
        let conn_id = ConnId::rnd();
        NatsClient::new(url, conn_id).connect("admin", &pass).await
    }

    #[tokio::test]
    async fn validate_payload_size() -> BoxedResult<()> {
        let client = connect().await?;

        // Test with a payload within the size limit
        let small_payload = Bytes::from(vec![0; 100]);
        assert!(client
            .validate_payload(&small_payload, "test.subject")
            .is_ok());

        // Test with a payload exceeding the size limit
        let max_payload_size =
            client.conn.clone().unwrap().server_info().max_payload;

        let large_payload = Bytes::from(vec![0; max_payload_size + 1]);
        assert!(client
            .validate_payload(&large_payload, "test.subject")
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn create_stream_and_consumer() -> BoxedResult<()> {
        let client = connect().await?;
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

use async_nats::jetstream::context::{Publish, Streams};
use bytes::Bytes;
use tracing::info;

use super::{types::*, ClientOpts, NatsError};
use crate::types::BoxedResult;

#[derive(Debug, Clone)]
pub struct NatsClient {
    pub nats_client: async_nats::Client,
    pub jetstream: JetStreamContext,
    pub(crate) opts: ClientOpts,
}

impl NatsClient {
    pub async fn connect(opts: ClientOpts) -> Result<Self, NatsError> {
        let nats_client = opts.to_owned().connect().await?;
        let jetstream = async_nats::jetstream::new(nats_client.to_owned());
        info!("Connected to NATS server at {}", &opts.url);

        Ok(Self {
            opts: opts.to_owned(),
            nats_client,
            jetstream,
        })
    }

    pub fn validate_payload(
        &self,
        payload: &Bytes,
        subject_name: &str,
    ) -> Result<&Self, NatsError> {
        let payload_size = payload.len();
        let conn = self.nats_client.clone();
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

    pub fn stream_name(&self, val: &str) -> String {
        format!("{}:stream:{val}", self.opts.prefix)
    }

    pub fn consumer_name(&self, val: &str) -> String {
        format!("{}:consumer:{val}", self.opts.prefix)
    }

    pub fn subject_name(&self, val: &str) -> String {
        format!("{}.{val}", self.opts.prefix)
    }

    pub async fn publish(
        &self,
        subject: String,
        payload: Bytes,
    ) -> BoxedResult<&Self> {
        let subject = self.subject_name(&subject);
        let payload = Publish::build().message_id(&subject).payload(payload);
        self.jetstream.send_publish(subject, payload).await?.await?;
        Ok(self)
    }

    pub fn prefix_subjects(&self, subjects: Vec<String>) -> Vec<String> {
        subjects
            .iter()
            .map(|s| format!("{}.{s}", self.opts.prefix))
            .collect()
    }

    pub async fn create_stream(
        &self,
        name: &str,
        config: JetStreamConfig,
    ) -> Result<AsyncNatsStream, NatsError> {
        let name = self.stream_name(name);
        self.jetstream
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
        self.jetstream.streams()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::prelude::*;
    async fn connect() -> Result<NatsClient, NatsError> {
        let host = "localhost:4222".to_string();
        let url = dotenvy::var("NATS_URL").unwrap_or(host);
        let opts = ClientOpts::admin_opts(url, ConnId::rnd());
        NatsClient::connect(opts).await
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
        let max_payload_size = client.nats_client.server_info().max_payload;
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

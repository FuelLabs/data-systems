use std::{sync::Arc, time::Duration};

use async_nats::connection;
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
        let conn = Arc::new(Self::create_conn(url, nkey).await?);
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

    pub async fn validate_payload(
        &self,
        payload: Bytes,
        subject_name: &str,
    ) -> Result<(), NatsError> {
        let payload_size = payload.len();
        let max_payload_size = self.conn.server_info().max_payload;
        if payload_size > max_payload_size {
            return Err(NatsError::PayloadTooLarge {
                subject: subject_name.to_string(),
                payload_size,
                max_payload_size,
            });
        }

        Ok(())
    }

    pub async fn create_conn(
        url: &str,
        nkey: Option<String>,
    ) -> Result<async_nats::Client, NatsError> {
        let options = async_nats::ConnectOptions::new()
            .connection_timeout(Duration::from_secs(30))
            .max_reconnects(10);

        if let Some(nkey) = nkey {
            async_nats::connect_with_options(&url, options.nkey(nkey)).await
        } else {
            async_nats::connect_with_options(&url, options).await
        }
        .map_err(|e| NatsError::ConnectError {
            url: url.to_owned(),
            source: e,
        })
    }

    pub async fn is_connected(&self) -> Result<bool, NatsError> {
        let conn_state = self.conn.connection_state();
        match conn_state {
            connection::State::Pending => {
                Err(NatsError::ConnectionPending(self.url.to_owned()))
            }
            connection::State::Disconnected => {
                Err(NatsError::ConnectionDisconnected(self.url.to_owned()))
            }
            connection::State::Connected => Ok(true),
        }
    }

    pub async fn create_stream(
        &self,
        name: &str,
        config: JetStreamConfig,
    ) -> Result<NatsStream, NatsError> {
        let name = format!("{}__stream:{}", self.conn_id, name);
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
        let name = format!("{}__consumer:{}", self.conn_id, name);
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

    pub async fn publish(
        &self,
        subject: Subject,
        payload: Bytes,
    ) -> BoxedResult<()> {
        let subject_prefixed = subject.with_prefix(&self.conn_id);
        let _ = self
            .validate_payload(payload.clone(), &subject_prefixed)
            .await;

        let context = self.jetstream.as_ref();
        let ack_future = context.publish(subject_prefixed, payload).await?;
        ack_future.await?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn setup_container() -> Result<
        testcontainers::ContainerAsync<testcontainers::GenericImage>,
        testcontainers::TestcontainersError,
    > {
        use testcontainers::{
            core::{IntoContainerPort, WaitFor},
            runners::AsyncRunner,
            GenericImage,
            ImageExt,
        };

        std::env::set_var("TESTCONTAINERS_COMMAND", "keep");

        GenericImage::new("nats", "latest")
            .with_exposed_port(4222.tcp())
            .with_wait_for(WaitFor::message_on_stderr("Server is ready"))
            .with_wait_for(WaitFor::message_on_stderr(
                "Listening for client connections on 0.0.0.0:4222",
            ))
            // This is needed to avoid intermittent failures
            .with_wait_for(WaitFor::millis(2000))
            .with_cmd(["-m", "8222", "--name", "fuel-core-nats-server", "--js"])
            .start()
            .await
    }

    #[cfg(test)]
    pub async fn connect_with_testcontainer() -> anyhow::Result<(
        Self,
        impl std::future::Future<Output = anyhow::Result<()>>,
    )> {
        let container = Self::setup_container().await?;
        let host = container.get_host().await?;
        let host_port = container.get_host_port_ipv4(4222).await?;
        let url = format!("{host}:{host_port}");
        let client = NatsClient::connect(url.as_str(), "test", None).await?;

        Ok((client, async move {
            container.stop().await?;
            container.rm().await?;
            Ok(())
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn create_connection() -> BoxedResult<()> {
        let (client, cleanup) =
            NatsClient::connect_with_testcontainer().await?;
        assert!(client.is_connected().await?);
        cleanup.await?;
        Ok(())
    }

    #[tokio::test]
    async fn validate_payload_size() -> BoxedResult<()> {
        let (client, cleanup) =
            NatsClient::connect_with_testcontainer().await?;

        // Test with a payload within the size limit
        let small_payload = Bytes::from(vec![0; 100]);
        assert!(client
            .validate_payload(small_payload, "test.subject")
            .await
            .is_ok());

        // Test with a payload exceeding the size limit
        let max_payload_size = client.conn.server_info().max_payload;
        let large_payload = Bytes::from(vec![0; max_payload_size + 1]);
        assert!(client
            .validate_payload(large_payload, "test.subject")
            .await
            .is_err());

        cleanup.await?;
        Ok(())
    }

    #[tokio::test]
    async fn create_stream_and_consumer() -> BoxedResult<()> {
        let (client, cleanup) =
            NatsClient::connect_with_testcontainer().await?;

        let mut stream = client
            .create_stream("test_stream", JetStreamConfig::default())
            .await?;

        let stream_info = stream.info().await?;
        let name = stream_info.config.name.clone();
        assert_eq!(name, "test__stream:test_stream");

        let mut consumer = client
            .create_pull_consumer("test_consumer", &stream, None)
            .await?;

        let consumer_info = consumer.info().await?;
        let name = consumer_info.config.durable_name.clone().unwrap();
        assert_eq!(name, "test__consumer:test_consumer");

        cleanup.await?;
        Ok(())
    }
}

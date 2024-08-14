use async_nats::{
    error,
    jetstream::{
        context::{CreateKeyValueErrorKind, CreateStreamErrorKind},
        kv,
        stream,
    },
};
use bytes::Bytes;
use tracing::info;

use super::{types::*, ClientOpts, NatsError, NatsNamespace};

#[derive(Debug, Clone)]
pub struct NatsClient {
    pub nats_client: async_nats::Client,
    pub jetstream: JetStreamContext,
    pub namespace: NatsNamespace,
    pub opts: ClientOpts,
}

impl NatsClient {
    pub async fn connect(opts: ClientOpts) -> Result<Self, NatsError> {
        let url = opts.to_owned().url;
        let nats_client = opts
            .connect_opts()
            .connect(&url)
            .await
            .map_err(|e| NatsError::ConnectionError { url, source: e })?;

        let jetstream = async_nats::jetstream::new(nats_client.to_owned());
        info!("Connected to NATS server at {}", &opts.url);

        Ok(Self {
            nats_client,
            jetstream,
            opts: opts.to_owned(),
            namespace: opts.namespace,
        })
    }

    #[allow(dead_code)]
    fn validate_payload(
        &self,
        payload: &Bytes,
        subject_name: &str,
    ) -> Result<&Self, NatsError> {
        let payload_size = payload.len();
        let conn = self.nats_client.clone();
        let max_payload_size = conn.server_info().max_payload;
        if payload_size > max_payload_size {
            return Err(NatsError::PayloadTooLarge {
                subject_name: subject_name.to_string(),
                payload_size,
                max_payload_size,
            });
        }

        Ok(self)
    }

    pub async fn get_or_create_store(
        &self,
        options: kv::Config,
    ) -> Result<kv::Store, error::Error<CreateKeyValueErrorKind>> {
        let bucket = options.bucket.clone();
        let store = self.jetstream.get_key_value(&bucket).await;
        let store = match store {
            Ok(store) => store,
            Err(_) => self.jetstream.create_key_value(options).await?,
        };

        Ok(store)
    }

    pub async fn get_or_create_stream(
        &self,
        options: stream::Config,
    ) -> Result<stream::Stream, error::Error<CreateStreamErrorKind>> {
        self.jetstream.get_or_create_stream(options).await
    }

    pub fn opts(&self) -> &ClientOpts {
        &self.opts
    }

    pub fn jetstream(&self) -> &JetStreamContext {
        &self.jetstream
    }

    pub fn state(&self) -> ConnectionState {
        self.nats_client.connection_state()
    }

    pub fn nats_client(&self) -> &AsyncNatsClient {
        &self.nats_client
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    async fn connect() -> Result<NatsClient, NatsError> {
        let host = "localhost:4222".to_string();
        let url = dotenvy::var("NATS_URL").unwrap_or(host);
        let opts = ClientOpts::admin_opts(&url);
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
}

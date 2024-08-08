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

    pub async fn create_store(
        &self,
        bucket: &str,
        config: Option<NatsStoreConfig>,
    ) -> Result<NatsStore, NatsError> {
        let bucket = self.namespace.store_name(bucket);
        let store = self.jetstream.get_key_value(&bucket).await;
        let store = match store {
            Ok(store) => store,
            Err(_) => self
                .jetstream
                .create_key_value(NatsStoreConfig {
                    bucket: bucket.to_owned(),
                    storage: NatsStorageType::File,
                    compression: true,
                    ..config.unwrap_or_default()
                })
                .await
                .map_err(|s| NatsError::CreateStoreFailed {
                    name: bucket,
                    source: s,
                })?,
        };

        Ok(store)
    }
}

#[cfg(test)]
mod test {
    use std::str::from_utf8;

    use futures_util::StreamExt;
    use pretty_assertions::assert_eq;

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

    #[tokio::test]
    async fn creating_store() -> BoxedResult<()> {
        let client = connect().await?;
        let store = client.create_store("test_store", None).await?;
        assert_eq!(&store.stream_name, "KV_fuel_test_store");

        let mut watch = store.watch("test.*").await?;
        store.put("test.1", "data".into()).await?;

        let entry = watch.next().await.unwrap()?;
        let entry_value = from_utf8(&entry.value)?;
        assert_eq!(entry.bucket, client.namespace.store_name("test_store"));
        assert_eq!(entry.key, "test.1");
        assert_eq!(entry_value, "data");

        Ok(())
    }
}

use async_nats::{
    error,
    jetstream::{
        context::{CreateKeyValueErrorKind, CreateStreamErrorKind},
        kv,
        stream,
    },
};
use tracing::info;

use super::{types::*, NatsClientOpts, NatsError, NatsNamespace};

#[derive(Debug, Clone)]
/// NatsClient is a wrapper around the NATS client that provides additional functionality
/// geared towards fuel-streaming use-cases
pub struct NatsClient {
    pub nats_client: async_nats::Client,
    pub jetstream: JetStreamContext,
    pub namespace: NatsNamespace,
    pub opts: NatsClientOpts,
}

impl NatsClient {
    pub async fn connect(opts: &NatsClientOpts) -> Result<Self, NatsError> {
        let url = &opts.url;
        let namespace = opts.namespace.clone();
        let nats_client =
            opts.connect_opts().connect(&url).await.map_err(|e| {
                NatsError::ConnectionError {
                    url: url.to_string(),
                    source: e,
                }
            })?;

        let jetstream = async_nats::jetstream::new(nats_client.to_owned());
        info!("Connected to NATS server at {}", url);

        Ok(Self {
            nats_client,
            jetstream,
            opts: opts.to_owned(),
            namespace,
        })
    }

    pub async fn get_or_create_kv_store(
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

    pub fn is_connected(&self) -> bool {
        self.state() == ConnectionState::Connected
    }

    fn state(&self) -> ConnectionState {
        self.nats_client.connection_state()
    }
}

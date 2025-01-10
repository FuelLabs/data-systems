use std::sync::Arc;

use futures::{Stream, StreamExt};
use tracing::info;

use super::{types::*, NatsClientOpts, NatsError, NatsNamespace};

/// NatsClient is a wrapper around the NATS client that provides additional functionality
/// geared towards fuel-streaming use-cases
///
/// # Examples
///
/// Creating a new `NatsClient`:
///
/// ```no_run
/// use fuel_streams_nats::*;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let opts = NatsClientOpts::public_opts();
///     let client = NatsClient::connect(&opts).await?;
///     Ok(())
/// }
/// ```
///
/// Creating a key-value store:
///
/// ```no_run
/// use fuel_streams_nats::*;
/// use async_nats::jetstream::kv;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let opts = NatsClientOpts::public_opts();
///     let client = NatsClient::connect(&opts).await?;
///     let kv_config = kv::Config {
///         bucket: "my-bucket".into(),
///         ..Default::default()
///     };
///
///     let store = client.get_or_create_kv_store(kv_config).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct NatsClient {
    /// The underlying NATS client
    pub nats_client: async_nats::Client,
    /// The JetStream context for this client
    pub jetstream: JetStreamContext,
    /// The namespace used for this client
    pub namespace: NatsNamespace,
    /// The options used to create this client
    pub opts: NatsClientOpts,
}

impl NatsClient {
    pub async fn connect(opts: &NatsClientOpts) -> Result<Self, NatsError> {
        let url = &opts.get_url();
        let namespace = opts.namespace.clone();
        let nats_client =
            opts.connect_opts().connect(url).await.map_err(|e| {
                NatsError::ConnectionError {
                    url: url.to_string(),
                    source: e,
                }
            })?;

        let jetstream = match opts.domain.clone() {
            None => async_nats::jetstream::new(nats_client.clone()),
            Some(domain) => {
                async_nats::jetstream::with_domain(nats_client.clone(), domain)
            }
        };
        info!("Connected to NATS server at {}", url);

        Ok(Self {
            nats_client,
            jetstream,
            opts: opts.to_owned(),
            namespace,
        })
    }

    pub async fn publish(
        &self,
        subject: impl ToString,
        payload: impl Into<Vec<u8>>,
    ) -> Result<(), NatsError> {
        let subject = subject.to_string();
        self.nats_client
            .publish(subject, payload.into().into())
            .await?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        subject: impl ToString,
    ) -> Result<impl Stream<Item = Vec<u8>>, NatsError> {
        let subject = subject.to_string();
        let subscription = self.nats_client.subscribe(subject).await?;
        let subscription = subscription.then(|msg| async move {
            let payload = msg.payload;
            payload.to_vec()
        });
        Ok(subscription)
    }

    pub fn is_connected(&self) -> bool {
        self.state() == ConnectionState::Connected
    }

    fn state(&self) -> ConnectionState {
        self.nats_client.connection_state()
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }
}

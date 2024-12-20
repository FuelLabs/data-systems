use async_nats::{
    error,
    jetstream::{context::CreateKeyValueErrorKind, kv},
};
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
/// use fuel_streams_storage::nats::*;
/// use fuel_networks::FuelNetwork;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let opts = NatsClientOpts::new(FuelNetwork::Local);
///     let client = NatsClient::connect(&opts).await?;
///     Ok(())
/// }
/// ```
///
/// Creating a key-value store:
///
/// ```no_run
/// use fuel_streams_storage::nats::*;
/// use fuel_networks::FuelNetwork;
/// use async_nats::jetstream::kv;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let opts = NatsClientOpts::new(FuelNetwork::Local);
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

    pub fn is_connected(&self) -> bool {
        self.state() == ConnectionState::Connected
    }

    fn state(&self) -> ConnectionState {
        self.nats_client.connection_state()
    }
}

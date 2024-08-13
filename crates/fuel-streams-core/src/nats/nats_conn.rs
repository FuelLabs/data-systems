use super::{conn_stores::ConnStores, types::*, ClientOpts, NatsClient};

#[derive(Clone)]
pub struct NatsConn {
    pub client: NatsClient,
    pub stores: ConnStores,
}

impl NatsConn {
    #[cfg(feature = "test-helpers")]
    pub async fn connect(opts: ClientOpts) -> Result<Self, super::NatsError> {
        use fuel_data_parser::DataParser;

        let client = NatsClient::connect(opts).await?;
        let stores = ConnStores::new(&client, &DataParser::default()).await?;
        Ok(Self { client, stores })
    }

    pub fn opts(&self) -> &ClientOpts {
        &self.client.opts
    }

    pub fn jetstream(&self) -> &JetStreamContext {
        &self.client.jetstream
    }

    pub fn state(&self) -> ConnectionState {
        self.client.nats_client.connection_state()
    }

    pub fn nats_client(&self) -> &AsyncNatsClient {
        &self.client.nats_client
    }

    pub fn stores(&self) -> &ConnStores {
        &self.stores
    }
}

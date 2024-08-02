use super::{types::*, ClientOpts, ConnStreams, NatsClient, NatsError};

#[derive(Debug, Clone)]
pub struct NatsConn {
    pub client: NatsClient,
    pub streams: ConnStreams,
}

impl NatsConn {
    #[cfg(feature = "test-helpers")]
    pub async fn connect(opts: ClientOpts) -> Result<Self, NatsError> {
        let client = NatsClient::connect(opts).await?;
        let streams = ConnStreams::new(&client).await?;
        Ok(Self { streams, client })
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

    pub fn streams(&self) -> &ConnStreams {
        &self.streams
    }
}

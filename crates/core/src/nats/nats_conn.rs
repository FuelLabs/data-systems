use super::{types::*, ClientOpts, ConnId, ConnStreams, NatsClient, NatsError};

#[derive(Debug, Clone)]
pub struct NatsConn {
    client: NatsClient,
    streams: ConnStreams,
}

impl NatsConn {
    #[cfg(feature = "test_helpers")]
    pub async fn connect(opts: ClientOpts) -> Result<Self, NatsError> {
        let client = NatsClient::connect(opts).await?;
        let streams = ConnStreams::new(&client).await?;

        Ok(Self {
            streams,
            client: client.clone(),
        })
    }

    pub fn opts(&self) -> ClientOpts {
        self.client().opts.clone()
    }

    pub fn conn_id(&self) -> ConnId {
        self.client().opts.conn_id.clone()
    }

    pub fn url(&self) -> String {
        self.client().opts.url
    }

    pub fn client(&self) -> NatsClient {
        self.client.clone()
    }

    pub fn state(&self) -> ConnectionState {
        self.nats_client().connection_state()
    }

    pub fn nats_client(&self) -> AsyncNatsClient {
        self.client().nats_client.clone()
    }

    pub fn jetstream(&self) -> JetStreamContext {
        self.client().jetstream.clone()
    }

    pub fn streams(&self) -> ConnStreams {
        self.streams.clone()
    }
}

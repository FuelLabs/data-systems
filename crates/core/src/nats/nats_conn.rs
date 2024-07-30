use super::{ConnStreams, NatsClient, NatsError};

#[derive(Debug, Clone)]
pub struct NatsConn {
    pub client: NatsClient,
    pub streams: ConnStreams,
}

impl NatsConn {
    pub async fn new(
        conn_id: &str,
        nats_url: &str,
        nats_nkey: &str,
    ) -> Result<Self, NatsError> {
        let client = NatsClient::connect(nats_url, conn_id, nats_nkey).await?;
        let streams = ConnStreams::new(&client).await?;

        Ok(Self {
            streams,
            client: client.clone(),
        })
    }
}

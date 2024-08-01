use super::{ConnId, ConnStreams, NatsClient, NatsError};

#[derive(Debug, Clone)]
pub struct NatsConn {
    client: NatsClient,
    streams: ConnStreams,
}

impl NatsConn {
    async fn connect(
        url: &str,
        conn_id: ConnId,
        user: &str,
        pass: &str,
    ) -> Result<Self, NatsError> {
        let client = NatsClient::new(url, conn_id).connect(user, pass).await?;
        let streams = ConnStreams::new(&client).await?;

        Ok(Self {
            streams,
            client: client.clone(),
        })
    }

    #[cfg(feature = "test_helpers")]
    pub async fn as_admin(
        url: impl AsRef<str>,
        conn_id: ConnId,
    ) -> Result<Self, NatsError> {
        let pass = dotenvy::var("NATS_ADMIN_PASS").unwrap();
        Self::connect(url.as_ref(), conn_id, "admin", &pass).await
    }

    pub async fn as_public(
        url: impl AsRef<str>,
        conn_id: ConnId,
    ) -> Result<Self, NatsError> {
        let pass = dotenvy::var("NATS_PUBLIC_PASS").unwrap();
        Self::connect(url.as_ref(), conn_id, "public", &pass).await
    }

    pub fn client(&self) -> NatsClient {
        self.client.clone()
    }
    pub fn streams(&self) -> ConnStreams {
        self.streams.clone()
    }
}

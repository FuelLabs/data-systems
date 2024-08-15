use async_trait::async_trait;
use fuel_streams_core::prelude::*;

use super::ConnectionResult;

#[async_trait]
pub trait ClientConn: Clone + Send {
    async fn connect(url: impl ToString + Send) -> ConnectionResult<Self>;
    async fn with_opts(opts: &NatsClientOpts) -> ConnectionResult<Self>;
}

#[derive(Debug, Clone)]
pub struct Client {
    pub conn: NatsClient,
}

#[async_trait]
impl ClientConn for Client {
    async fn connect(url: impl ToString + Send) -> ConnectionResult<Self> {
        let opts = NatsClientOpts::new(url);
        let conn = NatsClient::connect(&opts).await?;
        Ok(Self { conn })
    }

    async fn with_opts(opts: &NatsClientOpts) -> ConnectionResult<Self> {
        let conn = NatsClient::connect(opts).await?;
        Ok(Self { conn })
    }
}

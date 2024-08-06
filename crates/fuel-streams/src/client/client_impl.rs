use async_trait::async_trait;
use fuel_streams_core::prelude::*;

use super::ConnectionResult;

#[async_trait]
pub trait ClientConn: Clone + Send {
    async fn connect(url: impl ToString + Send) -> ConnectionResult<Self>;
    async fn with_opts(opts: ClientOpts) -> ConnectionResult<Self>;
}

#[derive(Clone)]
pub struct Client {
    pub conn: NatsConn,
}

#[async_trait]
impl ClientConn for Client {
    async fn connect(url: impl ToString + Send) -> ConnectionResult<Self> {
        let opts = ClientOpts::new(url);
        let conn = NatsConn::connect(opts).await?;
        Ok(Self { conn })
    }

    async fn with_opts(opts: ClientOpts) -> ConnectionResult<Self> {
        let conn = NatsConn::connect(opts).await?;
        Ok(Self { conn })
    }
}

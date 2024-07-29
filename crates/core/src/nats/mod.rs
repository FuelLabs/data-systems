mod client;
mod errors;
mod streams;
mod subjects;
mod types;

pub use client::*;
pub use errors::*;
pub use streams::*;
pub use subjects::*;
pub use types::*;

#[derive(Debug, Clone)]
pub struct Nats {
    pub client: NatsClient,
    pub streams: Streams,
}

impl Nats {
    pub async fn new(
        conn_id: impl AsRef<str>,
        nats_url: impl AsRef<str>,
        nats_nkey: impl AsRef<str>,
    ) -> Result<Self, NatsError> {
        let client = NatsClient::connect(nats_url, conn_id, nats_nkey).await?;
        let streams = Streams::new(&client).await.unwrap();

        Ok(Self {
            client: client.clone(),
            streams,
        })
    }
}

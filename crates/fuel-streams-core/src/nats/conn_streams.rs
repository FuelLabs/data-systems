use super::{stream::Streamer, NatsClient, NatsError};
use crate::types::*;

#[derive(Debug, Clone)]
pub struct ConnStreams {
    pub blocks: Streamer<Block>,
    pub transactions: Streamer<Transaction>,
}

impl ConnStreams {
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let blocks = Streamer::<Block>::get_or_init(client, None).await?;
        let transactions =
            Streamer::<Transaction>::get_or_init(client, None).await?;
        Ok(Self {
            transactions,
            blocks,
        })
    }
}

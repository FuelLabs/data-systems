use fuel_streams_core::{
    nats::NatsClient,
    stream::Streamer,
    types::{Block, Transaction},
};

#[derive(Debug, Clone)]
pub struct Streams {
    pub blocks: Streamer<Block>,
    pub transactions: Streamer<Transaction>,
}

impl Streams {
    pub async fn new(client: &NatsClient) -> Self {
        let blocks = Streamer::<Block>::get_or_init(client).await;
        let transactions = Streamer::<Transaction>::get_or_init(client).await;
        Self {
            transactions,
            blocks,
        }
    }
}

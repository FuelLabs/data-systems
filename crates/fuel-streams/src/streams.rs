use fuel_streams_core::{
    nats::NatsClient,
    stream::Stream,
    types::{Block, Transaction},
};

#[derive(Debug, Clone)]
pub struct Streams {
    pub blocks: Stream<Block>,
    pub transactions: Stream<Transaction>,
}

impl Streams {
    pub async fn new(client: &NatsClient) -> Self {
        let blocks = Stream::<Block>::get_or_init(client).await;
        let transactions = Stream::<Transaction>::get_or_init(client).await;
        Self {
            transactions,
            blocks,
        }
    }
}

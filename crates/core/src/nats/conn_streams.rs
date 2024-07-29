use super::streams::{stream::Stream, BlockSubjects, TransactionSubjects};
use crate::nats::{NatsClient, NatsError};

#[derive(Debug, Clone)]
pub struct ConnStreams {
    pub blocks: Stream<BlockSubjects>,
    pub transactions: Stream<TransactionSubjects>,
}

impl ConnStreams {
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let transactions = Stream::<TransactionSubjects>::new(client).await?;
        let blocks = Stream::<BlockSubjects>::new(client).await?;

        Ok(Self {
            transactions,
            blocks,
        })
    }
}

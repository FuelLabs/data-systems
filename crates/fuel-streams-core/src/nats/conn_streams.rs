use super::streams::Stream;
use crate::{
    nats::{NatsClient, NatsError},
    types::{Block, Transaction},
};

#[derive(Clone)]
pub struct ConnStreams {
    pub blocks: Stream<Block>,
    pub transactions: Stream<Transaction>,
}

impl ConnStreams {
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let transactions = Stream::<Transaction>::new(client).await?;
        let blocks = Stream::<Block>::new(client).await?;

        Ok(Self {
            transactions,
            blocks,
        })
    }
}

#[cfg(feature = "test-helpers")]
impl ConnStreams {
    pub fn get_stream_list(&self) -> Vec<super::types::AsyncNatsStream> {
        vec![self.blocks.stream.clone(), self.transactions.stream.clone()]
    }

    pub async fn collect_subjects(&self) -> anyhow::Result<Vec<String>> {
        let streams = self.get_stream_list();
        let mut all_subjects = Vec::new();
        for mut stream in streams {
            let info = stream.info().await?;
            all_subjects.extend(info.config.subjects.clone());
        }

        Ok(all_subjects)
    }
}

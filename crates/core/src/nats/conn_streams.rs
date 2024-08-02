use crate::nats::{
    streams::Stream,
    subjects::{BlockSubjects, TransactionSubjects},
    NatsClient,
    NatsError,
};

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

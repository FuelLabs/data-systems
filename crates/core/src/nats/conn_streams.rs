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

#[cfg(any(test, feature = "test_helpers"))]
impl ConnStreams {
    pub fn get_stream_list(
        streams: &ConnStreams,
    ) -> Vec<super::types::AsyncNatsStream> {
        vec![
            streams.blocks.stream().clone(),
            streams.transactions.stream().clone(),
        ]
    }

    pub async fn collect_subjects(
        streams: Vec<super::types::AsyncNatsStream>,
    ) -> anyhow::Result<Vec<String>> {
        let mut all_subjects = Vec::new();
        for mut stream in streams {
            let info = stream.info().await?;
            all_subjects.extend(info.config.subjects.clone());
        }

        Ok(all_subjects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoxedResult;

    #[tokio::test]
    async fn has_streams_created() -> BoxedResult<()> {
        let client = NatsClient::connect_when_testing(None).await?;
        let _ = ConnStreams::new(&client).await.is_ok();
        Ok(())
    }

    #[tokio::test]
    async fn has_subjects_wildcards() -> BoxedResult<()> {
        let client = NatsClient::connect_when_testing(None).await?;
        let conn_id = client.clone().conn_id;
        let streams = ConnStreams::new(&client).await?;
        let stream_list = ConnStreams::get_stream_list(&streams);
        let all_subjects = ConnStreams::collect_subjects(stream_list).await?;

        assert_eq!(
            all_subjects,
            vec![
                format!("{conn_id}.blocks.*.*"),
                format!("{conn_id}.transactions.*.*.*.*.*"),
                format!("{conn_id}.by_id.transactions.*.*")
            ]
        );

        Ok(())
    }
}

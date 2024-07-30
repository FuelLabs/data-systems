use streams_core::nats::{ConnStreams, NatsClient};

pub struct TestStreamsBuilder {
    pub client: NatsClient,
    pub streams: ConnStreams,
    pub all_subjects: Vec<String>,
}

impl TestStreamsBuilder {
    pub async fn setup() -> anyhow::Result<Self> {
        let client = NatsClient::connect_when_testing(None).await?;
        let streams = ConnStreams::new(&client).await?;
        let stream_list = streams.get_stream_list();
        let all_subjects = ConnStreams::collect_subjects(stream_list).await?;

        Ok(Self {
            client,
            streams,
            all_subjects,
        })
    }
}

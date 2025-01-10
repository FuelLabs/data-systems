use fuel_streams_core::{subjects::*, types::Block, DeliverPolicy};
use fuel_streams_store::record::{DataEncoder, Record};
use fuel_streams_test::{create_multiple_test_data, setup_stream};
use futures::StreamExt;

const NATS_URL: &str = "nats://localhost:4222";

#[tokio::test]
async fn test_streaming_live_data() -> anyhow::Result<()> {
    let stream = setup_stream(NATS_URL).await?;
    let data = create_multiple_test_data(10, 0);

    tokio::spawn({
        let data = data.clone();
        let stream = stream.clone();
        async move {
            let subject = BlocksSubject::new().with_block_height(None);
            let mut subscriber = stream
                .subscribe(subject, DeliverPolicy::New)
                .await
                .enumerate();

            while let Some((index, record)) = subscriber.next().await {
                let record = record.unwrap();
                let expected_subject = data[index].0.parse();
                let expected_block = &data[index].1;
                let decoded_block = Block::decode(&record.1).await.unwrap();
                assert_eq!(record.0, expected_subject);
                assert_eq!(decoded_block, *expected_block);

                if index == data.len() - 1 {
                    break;
                }
            }
        }
    });

    for (subject, block) in data {
        let packet = block.to_packet(subject.arc()).arc();
        stream.publish(&packet).await?;
    }

    Ok(())
}

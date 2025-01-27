use fuel_streams_core::{server::DeliverPolicy, subjects::*, types::Block};
use fuel_streams_store::record::{DataEncoder, Record};
use fuel_streams_test::{
    create_multiple_records,
    create_random_db_name,
    setup_stream,
};
use futures::StreamExt;
use pretty_assertions::assert_eq;

const NATS_URL: &str = "nats://localhost:4222";

#[tokio::test]
async fn test_streaming_live_data() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let stream = setup_stream(NATS_URL, &prefix).await?;
    let data = create_multiple_records(10, 0);

    tokio::spawn({
        let data = data.clone();
        let stream = stream.clone();
        async move {
            let subject = BlocksSubject::new().with_height(None);
            let mut subscriber = stream
                .subscribe(subject, DeliverPolicy::New)
                .await
                .enumerate();

            while let Some((index, record)) = subscriber.next().await {
                let record = record.unwrap();
                let expected_block = &data[index].1;
                let decoded_block = Block::decode(&record.1).await.unwrap();
                assert_eq!(decoded_block, *expected_block);
                if index == data.len() - 1 {
                    break;
                }
            }
        }
    });

    for (subject, block) in data {
        let packet = block.to_packet(&subject);
        let subject = packet.subject_str();
        stream.publish(&subject, packet.value.into()).await?;
    }

    Ok(())
}

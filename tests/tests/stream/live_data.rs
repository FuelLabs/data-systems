use fuel_streams_core::subjects::*;
use fuel_streams_test::{create_multiple_test_data, prefix_fn, setup_stream};
use futures::StreamExt;

const NATS_URL: &str = "nats://localhost:4222";

#[tokio::test]
async fn test_streaming_live_data() -> anyhow::Result<()> {
    let stream = setup_stream(NATS_URL).await?;
    let (prefix, with_prefix) = prefix_fn();
    let name = with_prefix("test");
    let data = create_multiple_test_data(10, name);

    tokio::spawn({
        let data = data.clone();
        let stream = stream.clone();
        async move {
            let mut subscriber = stream
                .subscribe_live(format!("{}.>", prefix))
                .await
                .unwrap()
                .enumerate();

            while let Some((index, record)) = subscriber.next().await {
                let subject = data[index].0.clone().parse();
                let record = record.unwrap();
                assert_eq!(record.subject.to_string(), subject);
                assert_eq!(record.value, "test".as_bytes());
                if record.subject.to_string() == subject {
                    break;
                }
            }
        }
    });

    for (subject, record) in data {
        stream.publish(&record.to_packet(subject.parse())).await?;
    }

    Ok(())
}

use fuel_streams_core::prelude::*;
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

            while let Some((index, message)) = subscriber.next().await {
                println!("Received message: {:?}", message);
                let subject = data[index].0.clone().parse();
                assert_eq!(message.subject.to_string(), subject);
                assert_eq!(message.payload, "test".as_bytes());
                if message.subject.to_string() == subject {
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

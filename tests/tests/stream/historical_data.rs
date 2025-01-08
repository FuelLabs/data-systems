use fuel_streams_core::subjects::*;
use fuel_streams_test::{create_multiple_test_data, prefix_fn, setup_stream};
use futures::StreamExt;
use tokio::time::{timeout, Duration};

const NATS_URL: &str = "nats://localhost:4222";

#[tokio::test]
async fn test_subscribe_historical_records() -> anyhow::Result<()> {
    let stream = setup_stream(NATS_URL).await?;
    let (prefix, with_prefix) = prefix_fn();

    // Create and publish initial records
    let historical_data = create_multiple_test_data(5, with_prefix("base"));
    for (subject, record) in historical_data.clone() {
        stream.publish(&record.to_packet(subject.parse())).await?;
    }

    // Subscribe and collect historical records
    let mut subscriber = stream
        .subscribe_historical(format!("tests.{}.>", prefix))
        .await?;

    let mut received_records = Vec::new();
    while let Some(record) =
        timeout(Duration::from_secs(1), subscriber.next()).await?
    {
        received_records.push(record);
        if received_records.len() == 5 {
            break;
        }
    }

    // Verify records
    assert_eq!(
        received_records.len(),
        5,
        "Should receive all historical records"
    );
    for (index, record) in received_records.iter().enumerate() {
        let expected_subject = historical_data[index].0.clone().parse();
        assert_eq!(record.subject, expected_subject);
    }

    Ok(())
}

#[tokio::test]
async fn test_subscribe_live_records() -> anyhow::Result<()> {
    let stream = setup_stream(NATS_URL).await?;
    let (prefix, with_prefix) = prefix_fn();

    // Start subscription before publishing
    let mut subscriber = stream
        .subscribe_historical(format!("tests.{}.>", prefix))
        .await?;

    // Publish new records
    let live_data = create_multiple_test_data(3, with_prefix("live"));
    for (subject, record) in live_data.clone() {
        stream.publish(&record.to_packet(subject.parse())).await?;
    }

    // Collect live records
    let mut received_records = Vec::new();
    while let Some(record) =
        timeout(Duration::from_secs(1), subscriber.next()).await?
    {
        received_records.push(record);
        if received_records.len() == 3 {
            break;
        }
    }

    // Verify records
    assert_eq!(received_records.len(), 3, "Should receive all live records");
    for (index, record) in received_records.iter().enumerate() {
        let expected_subject = live_data[index].0.clone().parse();
        assert_eq!(record.subject, expected_subject);
    }

    Ok(())
}

#[tokio::test]
async fn test_subscribe_historical_and_live_ordering() -> anyhow::Result<()> {
    let stream = setup_stream(NATS_URL).await?;
    let (prefix, with_prefix) = prefix_fn();

    // Create and publish historical records
    let historical_data =
        create_multiple_test_data(3, with_prefix("historical"));
    for (subject, record) in historical_data.clone() {
        stream.publish(&record.to_packet(subject.parse())).await?;
    }

    // Start subscription
    let mut subscriber = stream
        .subscribe_historical(format!("tests.{}.>", prefix))
        .await?;

    // Publish live records
    let live_data = create_multiple_test_data(2, with_prefix("live"));
    for (subject, record) in live_data.clone() {
        stream.publish(&record.to_packet(subject.parse())).await?;
    }

    // Collect all records
    let mut received_records = Vec::new();
    while let Some(record) =
        timeout(Duration::from_secs(1), subscriber.next()).await?
    {
        received_records.push(record);
        if received_records.len() == historical_data.len() + live_data.len() {
            break;
        }
    }

    // Verify total count
    assert_eq!(
        received_records.len(),
        historical_data.len() + live_data.len(),
        "Should receive all records"
    );

    // Verify historical records come first
    for (index, record) in received_records
        .iter()
        .take(historical_data.len())
        .enumerate()
    {
        let expected_subject = historical_data[index].0.clone().parse();
        assert_eq!(record.subject, expected_subject);
    }

    // Verify live records come after
    for (index, record) in received_records
        .iter()
        .skip(historical_data.len())
        .enumerate()
    {
        let expected_subject = live_data[index].0.clone().parse();
        assert_eq!(record.subject, expected_subject);
    }

    Ok(())
}

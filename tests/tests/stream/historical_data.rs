use std::{sync::Arc, time::Duration};

use fuel_streams_core::{
    server::DeliverPolicy,
    subjects::*,
    types::{Block, MockBlock},
    Stream,
    StreamError,
};
use fuel_streams_domains::MockMsgPayload;
use fuel_streams_store::record::{Record, RecordPacket};
use fuel_streams_test::{
    close_db,
    create_multiple_records,
    create_random_db_name,
    insert_records,
    setup_stream,
};
use fuel_streams_types::BlockHeight;
use fuel_web_utils::api_key::{ApiKeyError, MockApiKeyRole};
use futures::StreamExt;
use pretty_assertions::assert_eq;
use tokio::time::sleep;

const NATS_URL: &str = "nats://localhost:4222";
const STORAGE_WAIT_TIME: Duration = Duration::from_millis(100);

async fn setup_test_environment(
    block_count: usize,
    start_height: u32,
) -> anyhow::Result<(
    String,
    Stream<Block>,
    Vec<(Arc<dyn IntoSubject>, Block, RecordPacket)>,
)> {
    let prefix = create_random_db_name();
    let stream = setup_stream(NATS_URL, &prefix).await?;
    let data = create_multiple_records(block_count, start_height, &prefix);
    let store = stream.store();
    insert_records(&store, &prefix, &data).await?;
    sleep(STORAGE_WAIT_TIME).await;
    Ok((prefix, stream, data))
}

async fn verify_historical_data(
    stream: Stream<Block>,
    data: Vec<(Arc<dyn IntoSubject>, Block, RecordPacket)>,
    start_block_height: BlockHeight,
) -> anyhow::Result<Vec<Arc<Block>>> {
    let subject = BlocksSubject::new();
    let role = MockApiKeyRole::admin().into_inner();
    let mut subscriber = stream
        .subscribe(
            subject,
            DeliverPolicy::FromBlock {
                block_height: start_block_height,
            },
            &role,
        )
        .await;

    let mut received_blocks = Vec::new();
    let expected_count = data.len() - start_block_height.into_inner() as usize;
    while let Some(record) = subscriber.next().await {
        let record = record.unwrap();
        let block = record.payload.as_block().unwrap();
        let block_index = block.height.into_inner() as usize;
        let expected_block = &data[block_index].1;
        assert_eq!(
            *block, *expected_block,
            "Block data mismatch at height {}",
            block.height
        );

        received_blocks.push(block.clone());
        if received_blocks.len() == expected_count {
            break;
        }
    }

    assert_eq!(
        received_blocks.len(),
        expected_count,
        "Received {} blocks, expected {}",
        received_blocks.len(),
        expected_count
    );

    Ok(received_blocks)
}

#[tokio::test]
async fn test_streaming_historical_data() -> anyhow::Result<()> {
    let (_, stream, data) = setup_test_environment(10, 0).await?;
    let start_block_height = BlockHeight::from(3);
    let handle = tokio::spawn({
        let stream = stream.clone();
        let data = data.clone();
        async move { verify_historical_data(stream, data, start_block_height).await }
    });

    let received_blocks = handle.await??;
    assert_eq!(
        received_blocks.len(),
        data.len() - start_block_height.into_inner() as usize,
        "Final block count verification failed"
    );

    close_db(&stream.store().db).await;
    Ok(())
}

#[tokio::test]
async fn test_streaming_historical_data_without_proper_role(
) -> anyhow::Result<()> {
    let (_, stream, _) = setup_test_environment(5, 0).await?;
    let role = MockApiKeyRole::web_client().into_inner();
    let subject = BlocksSubject::new();
    let start_block_height = BlockHeight::from(1);
    let mut subscriber = stream
        .subscribe(
            subject,
            DeliverPolicy::FromBlock {
                block_height: start_block_height,
            },
            &role,
        )
        .await;

    let result = subscriber.next().await;
    assert!(result.is_some(), "Expected an error response");
    let error = result.unwrap();
    assert!(error.is_err(), "Expected an error, got success");
    matches!(
        error.unwrap_err(),
        StreamError::ApiKey(ApiKeyError::ScopePermission(_))
    );

    close_db(&stream.store().db).await;
    Ok(())
}

async fn insert_custom_block(
    stream: &Stream<Block>,
    prefix: &str,
    height: BlockHeight,
) -> anyhow::Result<()> {
    let block = MockBlock::build(height.into());
    let subject = BlocksSubject::from(&block).dyn_arc();
    let msg_payload = MockMsgPayload::build(height.into(), prefix);
    let timestamps = msg_payload.timestamp();
    let packet = block.to_packet(&subject, timestamps).with_namespace(prefix);
    insert_records(&stream.store(), prefix, &[(subject, block, packet)])
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_streaming_historical_outside_limit() -> anyhow::Result<()> {
    // Create a random prefix for this test
    let prefix = create_random_db_name();
    let stream = setup_stream(NATS_URL, &prefix).await?;
    let old_block_height = BlockHeight::from(1);
    let new_block_height = BlockHeight::from(700);

    // Insert the old block and 4 more recent blocks
    insert_custom_block(&stream, &prefix, old_block_height).await?;
    let new_block_height_u32 = new_block_height.into_inner() as u32;
    let records = create_multiple_records(4, new_block_height_u32, &prefix);
    insert_records(&stream.store(), &prefix, &records).await?;

    // Use the builder role which has 600 historical block limit
    let role = MockApiKeyRole::builder().into_inner();
    let subject = BlocksSubject::new();

    // Try to subscribe and access the historical data
    let mut subscriber = stream
        .subscribe(
            subject,
            DeliverPolicy::FromBlock {
                block_height: old_block_height,
            },
            &role,
        )
        .await;

    // We should get an error when trying to access the first block
    let result = subscriber.next().await;
    assert!(result.is_some(), "Expected an error response");
    let error = result.unwrap();
    assert!(error.is_err(), "Expected an error, got success");

    // Check for the correct error type - should be HistoricalLimitExceeded
    match error.unwrap_err() {
        StreamError::ApiKey(ApiKeyError::HistoricalLimitExceeded(limit)) => {
            assert_eq!(limit, "600", "Expected limit to be 600");
        }
        err => {
            panic!("Expected HistoricalLimitExceeded error, got: {:?}", err)
        }
    }

    close_db(&stream.store().db).await;
    Ok(())
}

#[tokio::test]
async fn test_streaming_historical_with_no_limit() -> anyhow::Result<()> {
    // Create a random prefix for this test
    let prefix = create_random_db_name();
    let stream = setup_stream(NATS_URL, &prefix).await?;
    let block_height = BlockHeight::from(1);
    let new_block_height = BlockHeight::from(700);

    // Insert the old block and 4 more recent blocks
    insert_custom_block(&stream, &prefix, block_height).await?;
    let new_block_height_u32 = new_block_height.into_inner() as u32;
    let records = create_multiple_records(4, new_block_height_u32, &prefix);
    insert_records(&stream.store(), &prefix, &records).await?;

    // Use the admin role which has no historical limit
    let role = MockApiKeyRole::admin().into_inner();
    let subject = BlocksSubject::new();

    // Try to subscribe and access the historical data
    let mut subscriber = stream
        .subscribe(subject, DeliverPolicy::FromBlock { block_height }, &role)
        .await;
    // We should be able to access the first block since we're using admin role
    let result = subscriber.next().await;
    assert!(result.is_some(), "Expected a block response");
    let response = result.unwrap();
    assert!(
        response.is_ok(),
        "Expected block to be retrieved successfully"
    );

    close_db(&stream.store().db).await;
    Ok(())
}

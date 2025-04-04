use std::{sync::Arc, time::Duration};

use fuel_streams_core::{
    server::DeliverPolicy,
    subjects::*,
    types::{Block, MockBlock},
    Stream,
    StreamError,
};
use fuel_streams_domains::{
    blocks::packets::DynBlockSubject,
    infra::{Db, RecordPacket},
    MockMsgPayload,
};
use fuel_streams_test::{
    close_db,
    create_multiple_records,
    create_random_db_name,
    insert_records,
    setup_db,
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
    db: &Arc<Db>,
    block_count: usize,
    start_height: BlockHeight,
) -> anyhow::Result<(
    String,
    Stream<Block>,
    Vec<(DynBlockSubject, Block, RecordPacket)>,
)> {
    let prefix = create_random_db_name();
    let stream = setup_stream(db, NATS_URL, &prefix).await?;
    let data = create_multiple_records(block_count, start_height, &prefix);
    insert_records(db, &prefix, &data).await?;
    sleep(STORAGE_WAIT_TIME).await;
    Ok((prefix, stream, data))
}

async fn verify_historical_data(
    stream: Stream<Block>,
    data: Vec<(DynBlockSubject, Block, RecordPacket)>,
    start_block_height: BlockHeight,
    total_streamed: usize,
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

    let base_height = data[0].1.height.into_inner() as usize;
    let mut received_blocks = Vec::new();
    while let Some(record) = subscriber.next().await {
        let record = record.unwrap();
        let block = record.payload.as_block().unwrap();
        let block_index =
            (block.height.into_inner() as usize).saturating_sub(base_height);
        let expected_block = &data[block_index].1;
        assert_eq!(
            *block, *expected_block,
            "Block data mismatch at height {}",
            block.height
        );

        received_blocks.push(block.clone());
        if received_blocks.len() == total_streamed {
            break;
        }
    }

    assert_eq!(
        received_blocks.len(),
        total_streamed,
        "Received {} blocks, expected {}",
        received_blocks.len(),
        total_streamed
    );

    Ok(received_blocks)
}

#[tokio::test]
async fn test_streaming_historical_data() -> anyhow::Result<()> {
    let db = setup_db().await?;
    let start_height = BlockHeight::random();
    let (_, stream, data) =
        setup_test_environment(&db, 10, start_height).await?;
    let stream_start = BlockHeight::from(start_height.into_inner() + 3);
    let received_blocks =
        verify_historical_data(stream, data, stream_start, 6).await?;
    assert_eq!(received_blocks.len(), 6, "Should receive exactly 6 blocks");
    close_db(&db).await;
    Ok(())
}

#[tokio::test]
async fn test_streaming_historical_data_without_proper_role(
) -> anyhow::Result<()> {
    let db = setup_db().await?;
    let (_, stream, _) = setup_test_environment(&db, 5, 0.into()).await?;
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

    close_db(&db).await;
    Ok(())
}

async fn insert_custom_block(
    db: &Arc<Db>,
    prefix: &str,
    height: BlockHeight,
) -> anyhow::Result<()> {
    let block = MockBlock::build(height);
    let msg_payload = MockMsgPayload::build(height, prefix);
    let timestamps = msg_payload.timestamp();
    let subject = DynBlockSubject::new(
        msg_payload.block_height(),
        msg_payload.block_producer(),
        &block.header.da_height,
    );
    let packet = subject
        .build_packet(&block, timestamps)
        .with_namespace(prefix);
    insert_records(db, prefix, &[(subject, block, packet)]).await?;
    Ok(())
}

#[tokio::test]
async fn test_streaming_historical_outside_limit() -> anyhow::Result<()> {
    // Create a random prefix for this test
    let db = setup_db().await?;
    let prefix = create_random_db_name();
    let stream = setup_stream(&db, NATS_URL, &prefix).await?;
    let block_height = BlockHeight::random();
    let old_block_height = BlockHeight::from(block_height.into_inner() - 700);

    // Insert the old block and 4 more recent blocks
    insert_custom_block(&db, &prefix, block_height).await?;
    let records = create_multiple_records(4, old_block_height, &prefix);
    let _ = insert_records(&db, &prefix, &records).await?;

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

    close_db(&db).await;
    Ok(())
}

#[tokio::test]
async fn test_streaming_historical_with_no_limit() -> anyhow::Result<()> {
    // Create a random prefix for this test
    let db = setup_db().await?;
    let prefix = create_random_db_name();
    let stream = setup_stream(&db, NATS_URL, &prefix).await?;
    let block_height = BlockHeight::from(1);
    let new_block_height = BlockHeight::from(700);

    // Insert the old block and 4 more recent blocks
    insert_custom_block(&db, &prefix, block_height).await?;
    let records = create_multiple_records(4, new_block_height, &prefix);
    insert_records(&db, &prefix, &records).await?;

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

    close_db(&db).await;
    Ok(())
}

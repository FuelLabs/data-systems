use std::{sync::Arc, time::Duration};

use fuel_streams_core::{
    server::DeliverPolicy,
    subjects::*,
    types::Block,
    Stream,
    StreamError,
};
use fuel_streams_store::record::RecordPacket;
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
    insert_records(store, &prefix, &data).await?;
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

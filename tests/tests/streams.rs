use std::collections::HashSet;

use futures_util::TryStreamExt;
use streams_core::{
    nats::{subjects, ConnStreams},
    types,
};
use streams_tests::TestStreamsBuilder;

#[tokio::test]
async fn has_conn_and_context_same_streams() -> types::BoxedResult<()> {
    let ctx = TestStreamsBuilder::setup().await?;
    let streams = ConnStreams::new(&ctx.client).await?;
    let stream_list = ConnStreams::get_stream_list(&streams);
    let mut jetstreams_streams = ctx.client.jetstream.streams();

    let mut found = HashSet::new();
    while let Some(stream) = jetstreams_streams.try_next().await? {
        found.insert(stream.config.name.clone());
    }
    for mut stream in stream_list {
        let info = stream.info().await?;
        assert!(
            found.contains(&info.config.name),
            "Stream {} not found in JetStream",
            info.config.name
        );
    }

    Ok(())
}

#[tokio::test]
async fn can_consume_stream_for_blocks() -> types::BoxedResult<()> {
    let ctx = TestStreamsBuilder::setup().await?;
    let stream = ctx.streams.blocks;
    let consumer = stream.create_pull_consumer(&ctx.client, None).await?;
    let subject = subjects::blocks::Blocks {
        producer: Some("0x000".to_string()),
        height: Some(100_u32),
    };

    // Checking consumer name created with consumer_from method
    stream
        .assert_consumer_name(&ctx.client, consumer.to_owned())
        .await?;

    stream
        .asset_message_from_subject(&ctx.client, consumer, subject)
        .await?;

    Ok(())
}

#[tokio::test]
async fn can_consume_stream_for_transactions() -> types::BoxedResult<()> {
    let ctx = TestStreamsBuilder::setup().await?;
    let stream = ctx.streams.transactions;
    let consumer = stream.create_pull_consumer(&ctx.client, None).await?;
    let subject = subjects::transactions::Transactions {
        height: Some(100_u32),
        tx_index: Some(1),
        tx_id: Some("0x000".to_string()),
        status: Some(types::TransactionStatus::Success),
        kind: Some(types::TransactionKind::Script),
    };

    stream
        .assert_consumer_name(&ctx.client, consumer.to_owned())
        .await?;

    stream
        .asset_message_from_subject(&ctx.client, consumer, subject)
        .await?;

    Ok(())
}

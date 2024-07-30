use std::collections::HashSet;

use futures_util::{StreamExt, TryStreamExt};
use streams_core::{
    nats::{subjects, ConnStreams, Subject},
    types,
};
use streams_tests::TestStreamsBuilder;

#[tokio::test]
async fn has_conn_and_context_same_streams() -> types::BoxedResult<()> {
    let ctx = TestStreamsBuilder::setup().await?;
    let streams = ConnStreams::new(&ctx.client).await?;
    let stream_list = streams.get_stream_list();
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
    let stream = ctx.streams.clone().blocks;
    let consumer = stream.create_pull_consumer(&ctx.client, None).await?;
    let subject = subjects::blocks::Blocks {
        producer: Some("0x000".to_string()),
        height: Some(100_u32),
    };

    stream
        .assert_consumer_name(&ctx.client, consumer.to_owned())
        .await?;

    let payload_data = "data";
    ctx.client
        .publish(subject.parse(), payload_data.into())
        .await?;

    let messages = consumer.messages().await?.take(10);
    stream
        .assert_messages_consumed(messages, subject, payload_data)
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

    let payload_data = "data";
    ctx.client
        .publish(subject.parse(), payload_data.into())
        .await?;

    let messages = consumer.messages().await?.take(10);
    stream
        .assert_messages_consumed(messages, subject, payload_data)
        .await?;

    Ok(())
}

#[tokio::test]
async fn consume_stream_with_dedup() -> types::BoxedResult<()> {
    let ctx = TestStreamsBuilder::setup().await?;
    let stream = ctx.streams.blocks;
    let consumer = stream.create_pull_consumer(&ctx.client, None).await?;
    let subject = subjects::blocks::Blocks {
        producer: Some("0x000".to_string()),
        height: Some(100_u32),
    };

    let payload_data = "data";
    let parsed = subject.parse();
    for _ in 0..100 {
        ctx.client
            .publish(parsed.to_owned(), payload_data.into())
            .await
            .is_ok();
    }

    let messages = consumer.messages().await?.take(1);
    let mut messages = stream
        .assert_messages_consumed(messages, subject, payload_data)
        .await?;

    // assert we only consumed one single message and the repeated ones were deduplicated by nats
    assert!(messages.next().await.transpose().ok().flatten().is_none());

    Ok(())
}

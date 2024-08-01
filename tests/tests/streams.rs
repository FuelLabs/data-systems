use futures_util::StreamExt;
use streams_core::prelude::*;
use streams_tests::TestSetupBuilder;

#[tokio::test]
async fn public_user_cannot_create_streams() {
    assert!(TestSetupBuilder::as_public(None).await.is_err());
}

#[tokio::test]
async fn public_user_can_access_streams_after_created() -> BoxedResult<()> {
    let ctx = TestSetupBuilder::as_admin(None).await?;
    assert!(TestSetupBuilder::as_public(Some(ctx.conn_id()))
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn can_consume_stream_for_blocks() -> BoxedResult<()> {
    let ctx = TestSetupBuilder::as_admin(None).await?;
    let client = ctx.conn.client();
    let stream = ctx.conn.streams().blocks();
    let consumer = stream.create_pull_consumer(&client, None).await?;
    let subject = subjects::blocks::Blocks {
        producer: Some("0x000".to_string()),
        height: Some(100_u32),
    };

    stream
        .assert_consumer_name(&client, consumer.to_owned())
        .await?;

    let payload_data = "data";
    client.publish(subject.parse(), payload_data.into()).await?;

    let messages = consumer.messages().await?.take(10);
    stream
        .assert_messages_consumed(messages, subject, payload_data)
        .await?;

    Ok(())
}

#[tokio::test]
async fn can_consume_stream_for_transactions() -> BoxedResult<()> {
    let ctx = TestSetupBuilder::as_admin(None).await?;
    let client = ctx.conn.client();
    let stream = ctx.conn.streams().transactions();
    let consumer = stream.create_pull_consumer(&client, None).await?;
    let subject = subjects::transactions::Transactions {
        height: Some(100_u32),
        tx_index: Some(1),
        tx_id: Some("0x000".to_string()),
        status: Some(TransactionStatus::Success),
        kind: Some(TransactionKind::Script),
    };

    stream
        .assert_consumer_name(&client, consumer.to_owned())
        .await?;

    let payload_data = "data";
    client.publish(subject.parse(), payload_data.into()).await?;

    let messages = consumer.messages().await?.take(10);
    stream
        .assert_messages_consumed(messages, subject, payload_data)
        .await?;

    Ok(())
}

#[tokio::test]
async fn consume_stream_with_dedup() -> BoxedResult<()> {
    let ctx = TestSetupBuilder::as_admin(None).await?;
    let client = ctx.conn.client();
    let stream = ctx.conn.streams().blocks();
    let consumer = stream.create_pull_consumer(&client, None).await?;
    let subject = subjects::blocks::Blocks {
        producer: Some("0x000".to_string()),
        height: Some(100_u32),
    };

    let payload_data = "data";
    let parsed = subject.parse();
    for _ in 0..100 {
        client
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

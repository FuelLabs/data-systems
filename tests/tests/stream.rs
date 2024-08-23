use fuel_streams::prelude::*;
use fuel_streams_core::prelude::*;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use streams_tests::{publish_blocks, publish_transactions, server_setup};

#[tokio::test]
async fn blocks_streams_subscribe() {
    let (conn, _) = server_setup().await.unwrap();
    let client = Client::with_opts(&conn.opts).await.unwrap();
    let stream = fuel_streams::Stream::<Block>::new(&client).await;
    let producer = Some(Address::zeroed());
    let items = publish_blocks(stream.stream(), producer).unwrap();

    let mut sub = stream.subscribe().await.unwrap().enumerate();
    while let Some((i, bytes)) = sub.next().await {
        let decoded_msg = Block::decode_raw(bytes).await;
        let (subject, block) = items[i].to_owned();
        let height = *decoded_msg.data.header().consensus().height;

        assert_eq!(decoded_msg.subject, subject.parse());
        assert_eq!(decoded_msg.data, block);
        assert_eq!(height, i as u32);
        if i == 9 {
            break;
        }
    }
}

#[tokio::test]
async fn blocks_streams_subscribe_with_config() {
    let (conn, _) = server_setup().await.unwrap();
    let client = Client::with_opts(&conn.opts).await.unwrap();
    let mut stream = fuel_streams::Stream::<Block>::new(&client).await;
    let producer = Some(Address::zeroed());

    // publishing 10 blocks
    publish_blocks(stream.stream(), producer).unwrap();

    // filtering by producer 0x000 and height 5
    let filter = Filter::<BlocksSubject>::build()
        .with_producer(Some(Address::zeroed()))
        .with_height(Some(5.into()));

    // creating subscription
    let mut sub = stream
        .with_filter(filter)
        .subscribe_with_config(StreamConfig::default())
        .await
        .unwrap()
        .take(10);

    // result should be just 1 single message with height 5
    while let Some(message) = sub.next().await {
        let message = message.unwrap();
        let decoded_msg =
            Block::decode_raw(message.payload.clone().into()).await;
        let height = *decoded_msg.data.header().consensus().height;
        assert_eq!(height, 5);
        if height == 5 {
            break;
        }
    }
}

#[tokio::test]
async fn transactions_streams_subscribe() {
    let (conn, _) = server_setup().await.unwrap();
    let client = Client::with_opts(&conn.opts).await.unwrap();
    let stream = fuel_streams::Stream::<Transaction>::new(&client).await;

    let mock_block = MockBlock::build(1);
    let items = publish_transactions(stream.stream(), &mock_block).unwrap();

    let mut sub = stream.subscribe().await.unwrap().enumerate();
    while let Some((i, bytes)) = sub.next().await {
        let decoded_msg = Transaction::decode_raw(bytes.to_vec()).await;

        let (_, transaction) = items[i].to_owned();
        assert_eq!(decoded_msg.data, transaction);
        if i == 9 {
            break;
        }
    }
}

#[tokio::test]
async fn transactions_streams_subscribe_with_config() {
    let (conn, _) = server_setup().await.unwrap();
    let client = Client::with_opts(&conn.opts).await.unwrap();
    let mut stream = fuel_streams::Stream::<Transaction>::new(&client).await;

    // publishing 10 transactions
    let mock_block = MockBlock::build(5);
    let items = publish_transactions(stream.stream(), &mock_block).unwrap();

    // filtering by transaction on block with height 5
    let filter =
        Filter::<TransactionsSubject>::build().with_height(Some(5.into()));

    // creating subscription
    let mut sub = stream
        .with_filter(filter)
        .subscribe_with_config(StreamConfig::default())
        .await
        .unwrap()
        .take(10)
        .enumerate();

    // result should be 10 transactions messages
    while let Some((i, message)) = sub.next().await {
        let message = message.unwrap();
        let payload = message.payload.clone().into();
        let decoded_msg = Transaction::decode(payload).await;

        println!("{}", &message.subject);
        let (_, transaction) = items[i].to_owned();
        assert_eq!(decoded_msg, transaction);
        if i == 9 {
            break;
        }
    }
}

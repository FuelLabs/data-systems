use fuel_streams::prelude::*;
use fuel_streams_core::prelude::*;
use futures::{future::try_join_all, StreamExt};
use pretty_assertions::assert_eq;
use streams_tests::{publish_blocks, publish_transactions, server_setup};

#[tokio::test]
async fn blocks_streams_subscribe() {
    let (_, _, client) = server_setup().await.unwrap();
    let stream = fuel_streams::Stream::<Block>::new(&client).await;
    let producer = Some(Address::zeroed());
    let items = publish_blocks(stream.stream(), producer, None).unwrap().0;

    let mut sub = stream.subscribe_raw().await.unwrap().enumerate();

    while let Some((i, bytes)) = sub.next().await {
        let decoded_msg = Block::decode_raw(bytes).unwrap();
        let (subject, block) = items[i].to_owned();
        let height = decoded_msg.payload.height;

        assert_eq!(decoded_msg.subject, subject.parse());
        assert_eq!(decoded_msg.payload, block);
        assert_eq!(height, i as u32);
        if i == 9 {
            break;
        }
    }
}

#[tokio::test]
async fn blocks_streams_subscribe_with_filter() {
    let (_, _, client) = server_setup().await.unwrap();
    let mut stream = fuel_streams::Stream::<Block>::new(&client).await;
    let producer = Some(Address::zeroed());

    // publishing 10 blocks
    publish_blocks(stream.stream(), producer, None).unwrap();

    // filtering by producer 0x000 and height 5
    let filter = Filter::<BlocksSubject>::build()
        .with_producer(Some(Address::zeroed()))
        .with_height(Some(5.into()));

    // creating subscription
    let mut sub = stream
        .with_filter(filter)
        .subscribe_raw_with_config(StreamConfig::default())
        .await
        .unwrap()
        .take(10);

    // result should be just 1 single message with height 5
    while let Some(bytes) = sub.next().await {
        let decoded_msg = Block::decode_raw(bytes).unwrap();
        let height = decoded_msg.payload.height;
        assert_eq!(height, 5);
        if height == 5 {
            break;
        }
    }
}

#[tokio::test]
async fn transactions_streams_subscribe() {
    let (_, _, client) = server_setup().await.unwrap();
    let stream = fuel_streams::Stream::<Transaction>::new(&client).await;

    let mock_block = MockBlock::build(1);
    let items = publish_transactions(stream.stream(), &mock_block, None)
        .unwrap()
        .0;

    let mut sub = stream.subscribe_raw().await.unwrap().enumerate();
    while let Some((i, bytes)) = sub.next().await {
        let decoded_msg = Transaction::decode_raw(bytes).unwrap();

        let (_, transaction) = items[i].to_owned();
        assert_eq!(decoded_msg.payload, transaction);
        if i == 9 {
            break;
        }
    }
}

#[tokio::test]
async fn transactions_streams_subscribe_with_filter() {
    let (_, _, client) = server_setup().await.unwrap();
    let mut stream = fuel_streams::Stream::<Transaction>::new(&client).await;

    // publishing 10 transactions
    let mock_block = MockBlock::build(5);
    let items = publish_transactions(stream.stream(), &mock_block, None)
        .unwrap()
        .0;

    // filtering by transaction on block with height 5
    let filter = Filter::<TransactionsSubject>::build()
        .with_block_height(Some(5.into()));

    // creating subscription
    let mut sub = stream
        .with_filter(filter)
        .subscribe_raw_with_config(StreamConfig::default())
        .await
        .unwrap()
        .take(10)
        .enumerate();

    // result should be 10 transactions messages
    while let Some((i, bytes)) = sub.next().await {
        let decoded_msg = Transaction::decode(bytes).unwrap();

        let (_, transaction) = items[i].to_owned();
        assert_eq!(decoded_msg, transaction);
        if i == 9 {
            break;
        }
    }
}

#[tokio::test]
async fn multiple_subscribers_same_subject() {
    let (_, _, client) = server_setup().await.unwrap();
    let stream = fuel_streams::Stream::<Block>::new(&client).await;
    let producer = Some(Address::zeroed());
    let items = publish_blocks(stream.stream(), producer.clone(), None)
        .unwrap()
        .0;

    let clients_count = 100;
    let done_signal = 99;
    let mut handles = Vec::new();
    for _ in 0..clients_count {
        let stream = stream.clone();
        let items = items.clone();
        handles.push(tokio::spawn(async move {
            let mut sub = stream.subscribe_raw().await.unwrap().enumerate();
            while let Some((i, bytes)) = sub.next().await {
                let decoded_msg = Block::decode_raw(bytes).unwrap();
                let (subject, block) = items[i].to_owned();
                let height = decoded_msg.payload.height;

                assert_eq!(decoded_msg.subject, subject.parse());
                assert_eq!(decoded_msg.payload, block);
                assert_eq!(height, i as u32);
                if i == 9 {
                    return done_signal;
                }
            }
            done_signal + 1
        }));
    }

    let mut client_results = try_join_all(handles).await.unwrap();
    assert!(
        client_results.len() == clients_count,
        "must have all clients subscribed to one subject"
    );
    client_results.dedup();
    assert!(
        client_results.len() == 1,
        "all clients must have the same result"
    );
    assert!(
        client_results.first().cloned().unwrap() == done_signal,
        "all clients must have the same received the complete signal"
    );
}

#[tokio::test]
async fn multiple_subscribers_different_subjects() {
    let (_, _, client) = server_setup().await.unwrap();
    let producer = Some(Address::zeroed());
    let block_stream = fuel_streams::Stream::<Block>::new(&client).await;
    let block_items =
        publish_blocks(block_stream.stream(), producer.clone(), None)
            .unwrap()
            .0;

    let txs_stream = fuel_streams::Stream::<Transaction>::new(&client).await;
    let mock_block = MockBlock::build(1);
    let txs_items =
        publish_transactions(txs_stream.stream(), &mock_block, None)
            .unwrap()
            .0;

    let clients_count = 100;
    let done_signal = 99;
    let mut handles = Vec::new();
    for _ in 0..clients_count {
        // blocks stream
        let stream = block_stream.clone();
        let items = block_items.clone();
        handles.push(tokio::spawn(async move {
            let mut sub = stream.subscribe_raw().await.unwrap().enumerate();
            while let Some((i, bytes)) = sub.next().await {
                let decoded_msg = Block::decode_raw(bytes).unwrap();
                let (subject, block) = items[i].to_owned();
                let height = decoded_msg.payload.height;

                assert_eq!(decoded_msg.subject, subject.parse());
                assert_eq!(decoded_msg.payload, block);
                assert_eq!(height, i as u32);
                if i == 9 {
                    return done_signal;
                }
            }
            done_signal + 1
        }));

        // txs stream
        let stream = txs_stream.clone();
        let items = txs_items.clone();
        handles.push(tokio::spawn(async move {
            let mut sub = stream.subscribe_raw().await.unwrap().enumerate();
            while let Some((i, bytes)) = sub.next().await {
                let decoded_msg = Transaction::decode_raw(bytes).unwrap();
                let (_, transaction) = items[i].to_owned();
                assert_eq!(decoded_msg.payload, transaction);
                if i == 9 {
                    return done_signal;
                }
            }
            done_signal + 1
        }));
    }

    let mut client_results = try_join_all(handles).await.unwrap();
    assert!(
        client_results.len() == 2 * clients_count,
        "must have all clients subscribed to two subjects"
    );
    client_results.dedup();
    assert!(
        client_results.len() == 1,
        "all clients must have the same result"
    );
    assert!(
        client_results.first().cloned().unwrap() == done_signal,
        "all clients must have the same received the complete signal"
    );
}

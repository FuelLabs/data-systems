use std::time::Duration;

use fuel_streams_core::prelude::*;
use futures::StreamExt;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn public_user_cannot_create_stores() {
    let opts = ClientOpts::public_opts(NATS_URL)
        .with_namespace("test1")
        .with_timeout(1);
    assert!(NatsConn::connect(opts).await.is_err());
}

#[tokio::test]
async fn can_watch_blocks_store() {
    let opts = ClientOpts::admin_opts(NATS_URL);
    let conn = NatsConn::connect(opts).await.unwrap();
    let store = conn.stores.blocks;
    let producer = Some("0x000".into());
    let wildcard = BlocksSubject::wildcard(producer.clone(), None);

    let mut items = Vec::new();
    for i in 0..10 {
        let block_item = MockBlock::build();
        let subject = BlocksSubject {
            producer: producer.clone(),
            height: Some(i.into()),
        };
        items.push((subject, block_item));
    }

    tokio::task::spawn({
        let store = store.clone();
        let items = items.clone();
        async move {
            for item in items {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let payload = item.1.clone();
                store.upsert(&item.0, &payload).await.unwrap();
            }
        }
    });

    let mut watch = store.watch(&wildcard).await.unwrap().enumerate();
    while let Some((i, entry)) = watch.next().await {
        let entry = entry.unwrap();
        let (subject, block) = items[i].to_owned();
        assert_eq!(entry.key, store.subject_name(&subject.parse()));
        assert_eq!(Block::store_decode(&entry.value).unwrap(), block);
        if i == 9 {
            break;
        }
    }
}

#[tokio::test]
async fn can_watch_transactions_store() {
    let opts = ClientOpts::admin_opts(NATS_URL);
    let conn = NatsConn::connect(opts).await.unwrap();
    let store = conn.stores.transactions;
    let wildcard = TransactionsSubject::all();

    let mut items = Vec::new();
    let mock_block = MockBlock::build();
    for i in 0..10 {
        let tx = MockTransaction::build();
        let subject = TransactionsSubject::from(tx.clone())
            .with_height(Some(mock_block.clone().into()))
            .with_tx_index(Some(i))
            .with_status(Some(TransactionStatus::Success));
        items.push((subject, tx));
    }

    tokio::task::spawn({
        let store = store.clone();
        let items = items.clone();
        async move {
            for item in items {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let payload = item.1.clone();
                store.upsert(&item.0, &payload).await.unwrap();
            }
        }
    });

    let mut watch = store.watch(wildcard).await.unwrap().enumerate();
    while let Some((i, entry)) = watch.next().await {
        let entry = entry.unwrap();
        let (subject, transaction) = items[i].to_owned();
        assert_eq!(entry.key, store.subject_name(&subject.parse()));
        assert_eq!(
            Transaction::store_decode(&entry.value).unwrap(),
            transaction
        );
        if i == 9 {
            break;
        }
    }
}

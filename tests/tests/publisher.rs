use std::{collections::HashMap, sync::Arc};

use fuel_core_importer::ImporterResult;
use fuel_core_types::blockchain::SealedBlock;
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{NatsClient, NatsClientOpts},
    prelude::*,
    types::ImportResult,
};
use fuel_streams_publisher::Publisher;
use tokio::sync::broadcast;

#[tokio::test]
async fn doesnt_publish_any_message_when_no_block_has_been_mined() {
    let (_, blocks_subscription) = broadcast::channel::<ImporterResult>(1);

    let publisher = Publisher::default_with_publisher(
        &nats_client().await,
        blocks_subscription,
    )
    .await
    .unwrap();
    let publisher = publisher.run().await.unwrap();

    assert!(publisher.get_streams().is_empty().await);
}

#[tokio::test]
async fn publishes_a_block_message_when_a_single_block_has_been_mined() {
    let (blocks_subscriber, blocks_subscription) =
        broadcast::channel::<ImporterResult>(1);

    let block = ImporterResult {
        shared_result: Arc::new(ImportResult::default()),
        changes: Arc::new(HashMap::new()),
    };
    let _ = blocks_subscriber.send(block);

    // manually drop blocks to ensure `blocks_subscription` completes
    let _ = blocks_subscriber.clone();
    drop(blocks_subscriber);

    let publisher = Publisher::default_with_publisher(
        &nats_client().await,
        blocks_subscription,
    )
    .await
    .unwrap();
    let publisher = publisher.run().await.unwrap();

    assert!(publisher
        .get_streams()
        .blocks
        .get_last_published(BlocksSubject::WILDCARD)
        .await
        .is_ok_and(|result| result.is_some()));
}

#[tokio::test]
async fn publishes_transaction_for_each_published_block() {
    let (blocks_subscriber, blocks_subscription) =
        broadcast::channel::<ImporterResult>(1);

    let mut block_entity = Block::default();
    *block_entity.transactions_mut() = vec![Transaction::default_test_tx()];

    // publish block
    let block = ImporterResult {
        shared_result: Arc::new(ImportResult {
            sealed_block: SealedBlock {
                entity: block_entity,
                ..Default::default()
            },
            ..Default::default()
        }),
        changes: Arc::new(HashMap::new()),
    };
    let _ = blocks_subscriber.send(block);

    // manually drop blocks to ensure `blocks_subscription` completes
    let _ = blocks_subscriber.clone();
    drop(blocks_subscriber);

    let publisher = Publisher::default_with_publisher(
        &nats_client().await,
        blocks_subscription,
    )
    .await
    .unwrap();
    let publisher = publisher.run().await.unwrap();

    assert!(publisher
        .get_streams()
        .transactions
        .get_last_published(TransactionsSubject::WILDCARD)
        .await
        .is_ok_and(|result| result.is_some()));
}

async fn nats_client() -> NatsClient {
    const NATS_URL: &str = "nats://localhost:4222";
    let nats_client_opts =
        NatsClientOpts::admin_opts(NATS_URL).with_rdn_namespace();
    NatsClient::connect(&nats_client_opts)
        .await
        .expect("NATS connection failed")
}

mod fuel_core_helpers;

use std::sync::Arc;

pub use fuel_core_helpers::*;
use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::{stream::*, subjects::*, types::Block};
use fuel_streams_domains::blocks::{subjects::BlocksSubject, types::MockBlock};
use fuel_streams_store::{
    db::{Db, DbConnectionOpts, DbResult},
    record::Record,
    store::Store,
};
use rand::Rng;

// -----------------------------------------------------------------------------
// Setup
// -----------------------------------------------------------------------------

pub async fn setup_db() -> DbResult<Db> {
    let opts = DbConnectionOpts {
        pool_size: Some(10),
        ..Default::default()
    };
    Db::new(opts).await
}

pub async fn setup_store<R: Record>() -> DbResult<Store<R>> {
    let db = setup_db().await?;
    let store = Store::new(&db.arc());
    Ok(store)
}

pub async fn setup_message_broker(
    nats_url: &str,
) -> anyhow::Result<Arc<dyn MessageBroker>> {
    let broker = MessageBrokerClient::Nats.start(nats_url).await?;
    broker.setup().await?;
    Ok(broker)
}

pub async fn setup_stream(nats_url: &str) -> anyhow::Result<Stream<Block>> {
    let db = setup_db().await?;
    let nats_client = setup_message_broker(nats_url).await?;
    let stream = Stream::<Block>::get_or_init(&nats_client, &db.arc()).await;
    Ok(stream)
}

// -----------------------------------------------------------------------------
// Test data
// -----------------------------------------------------------------------------

pub fn create_test_data(height: u32) -> (BlocksSubject, Block) {
    let block = MockBlock::build(height);
    let subject = BlocksSubject::from(&block);
    (subject, block)
}

pub fn create_multiple_test_data(
    count: usize,
    start_height: u32,
) -> Vec<(BlocksSubject, Block)> {
    (0..count)
        .map(|idx| create_test_data(start_height + idx as u32))
        .collect()
}

pub async fn add_test_records(
    store: &Store<Block>,
    prefix: &str,
    records: &[(Arc<dyn IntoSubject>, Block)],
) -> anyhow::Result<()> {
    for (subject, block) in records {
        let packet = block.to_packet(subject.clone()).with_namespace(prefix);
        store.insert_record(&packet).await?;
    }
    Ok(())
}

pub fn create_random_db_name() -> String {
    format!("test_{}", rand::thread_rng().gen_range(0..1000000))
}

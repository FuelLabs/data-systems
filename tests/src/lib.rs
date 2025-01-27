mod fuel_core_helpers;

use std::sync::Arc;

pub use fuel_core_helpers::*;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::{stream::*, subjects::IntoSubject, types::Block};
use fuel_streams_domains::blocks::{
    subjects::BlocksSubject,
    types::MockBlock,
    BlockDbItem,
};
use fuel_streams_store::{
    db::{Db, DbConnectionOpts, DbResult},
    record::{DbTransaction, Record},
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

pub async fn setup_stream(
    nats_url: &str,
    prefix: &str,
) -> anyhow::Result<Stream<Block>> {
    let db = setup_db().await?;
    let broker = NatsMessageBroker::setup(nats_url, Some(prefix)).await?;
    let stream = Stream::<Block>::get_or_init(&broker, &db.arc()).await;
    Ok(stream)
}

// -----------------------------------------------------------------------------
// Test data
// -----------------------------------------------------------------------------

pub fn create_record(height: u32) -> (Arc<dyn IntoSubject>, Block) {
    let block = MockBlock::build(height);
    let subject = BlocksSubject::from(&block).dyn_arc();
    (subject, block)
}

pub fn create_multiple_records(
    count: usize,
    start_height: u32,
) -> Vec<(Arc<dyn IntoSubject>, Block)> {
    (0..count)
        .map(|idx| create_record(start_height + idx as u32))
        .collect()
}

pub async fn insert_records(
    store: &Store<Block>,
    prefix: &str,
    records: &[(Arc<dyn IntoSubject>, Block)],
) -> anyhow::Result<Vec<BlockDbItem>> {
    let mut final_records = vec![];
    for (subject, block) in records {
        let packet = block.to_packet(subject).with_namespace(prefix);
        let record = store.insert_record(&packet).await?;
        final_records.push(record);
    }
    Ok(final_records)
}

pub async fn insert_records_with_transaction(
    store: &Store<Block>,
    tx: &mut DbTransaction,
    prefix: &str,
    records: &[(Arc<dyn IntoSubject>, Block)],
) -> anyhow::Result<()> {
    let mut final_records = vec![];
    for (subject, block) in records {
        let packet = block.to_packet(subject).with_namespace(prefix);
        let record = store.insert_record_with_transaction(tx, &packet).await?;
        final_records.push(record);
    }
    Ok(())
}

pub fn create_random_db_name() -> String {
    format!("test_{}", rand::thread_rng().gen_range(0..1000000))
}

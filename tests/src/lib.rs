mod fuel_core_helpers;

use std::sync::Arc;

pub use fuel_core_helpers::*;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::{stream::*, subjects::IntoSubject, types::Block};
use fuel_streams_domains::{
    blocks::{subjects::BlocksSubject, types::MockBlock, BlockDbItem},
    MockMsgPayload,
};
use fuel_streams_store::{
    db::{Db, DbConnectionOpts, DbResult},
    record::{DbTransaction, Record, RecordPacket},
    store::Store,
};
use rand::Rng;

// -----------------------------------------------------------------------------
// Setup
// -----------------------------------------------------------------------------

pub async fn setup_db() -> DbResult<Arc<Db>> {
    let opts = DbConnectionOpts::default();
    Db::new(opts).await
}

pub async fn close_db(db: &Arc<Db>) {
    db.close().await;
}

pub async fn setup_store<R: Record>() -> DbResult<Store<R>> {
    let db = setup_db().await?;
    let store = Store::new(&db);
    Ok(store)
}

pub async fn setup_stream(
    nats_url: &str,
    prefix: &str,
) -> anyhow::Result<Stream<Block>> {
    let db = setup_db().await?;
    let broker = NatsMessageBroker::setup(nats_url, Some(prefix)).await?;
    let stream =
        Stream::<Block>::with_namespace(&broker, &db, prefix.to_string());
    Ok(stream)
}

// -----------------------------------------------------------------------------
// Test data
// -----------------------------------------------------------------------------

pub fn create_record(
    height: u32,
    prefix: &str,
) -> (Arc<dyn IntoSubject>, Block, RecordPacket) {
    let block = MockBlock::build(height);
    let subject = BlocksSubject::from(&block);
    let subject = subject.dyn_arc();
    let msg_payload = MockMsgPayload::build(height, prefix);
    let timestamp = msg_payload.timestamp();
    let packet = block.to_packet(&subject, timestamp).with_namespace(prefix);
    (subject, block, packet)
}

pub fn create_multiple_records(
    count: usize,
    start_height: u32,
    prefix: &str,
) -> Vec<(Arc<dyn IntoSubject>, Block, RecordPacket)> {
    (0..count)
        .map(|idx| create_record(start_height + idx as u32, prefix))
        .collect()
}

pub async fn insert_records(
    store: &Store<Block>,
    prefix: &str,
    records: &[(Arc<dyn IntoSubject>, Block, RecordPacket)],
) -> anyhow::Result<Vec<BlockDbItem>> {
    let mut final_records = vec![];
    for record in records {
        let packet = record.2.to_owned().with_namespace(prefix);
        let db_item: BlockDbItem = (&packet).try_into()?;
        let record = store.insert_record(&db_item).await?;
        final_records.push(record);
    }
    Ok(final_records)
}

pub async fn insert_records_with_transaction(
    store: &Store<Block>,
    tx: &mut DbTransaction,
    prefix: &str,
    records: &[(Arc<dyn IntoSubject>, Block, RecordPacket)],
) -> anyhow::Result<()> {
    let mut final_records = vec![];
    for record in records {
        let packet = record.2.to_owned().with_namespace(prefix);
        let db_item: BlockDbItem = (&packet).try_into()?;
        let record = store.insert_record_with_transaction(tx, &db_item).await?;
        final_records.push(record);
    }
    Ok(())
}

pub fn create_random_db_name() -> String {
    format!("test_{}", rand::rng().random_range(0..1000000))
}

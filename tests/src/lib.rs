mod fuel_core_helpers;

use std::sync::Arc;

pub use fuel_core_helpers::*;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::{stream::*, types::Block};
use fuel_streams_domains::{
    blocks::{packets::DynBlockSubject, types::MockBlock, BlockDbItem},
    infra::{
        db::{Db, DbConnectionOpts, DbResult, DbTransaction},
        record::RecordPacket,
        repository::Repository,
    },
    MockMsgPayload,
};
use fuel_streams_types::BlockHeight;
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

pub async fn setup_stream(
    db: &Arc<Db>,
    nats_url: &str,
    prefix: &str,
) -> anyhow::Result<Stream<Block>> {
    let broker = NatsMessageBroker::setup(nats_url, Some(prefix)).await?;
    let stream =
        Stream::<Block>::with_namespace(&broker, db, prefix.to_string());
    Ok(stream)
}

// -----------------------------------------------------------------------------
// Test data
// -----------------------------------------------------------------------------

pub fn create_record(
    height: BlockHeight,
    prefix: &str,
) -> (DynBlockSubject, Block, RecordPacket) {
    let block = MockBlock::build(height);
    let subject = DynBlockSubject::new(
        block.height,
        block.producer.clone(),
        &block.header.da_height,
    );
    let msg_payload = MockMsgPayload::build(height, prefix);
    let timestamp = msg_payload.timestamp();
    let packet = subject
        .build_packet(&block, timestamp)
        .with_namespace(prefix);
    (subject, block, packet)
}

pub fn create_multiple_records(
    count: usize,
    start_height: BlockHeight,
    prefix: &str,
) -> Vec<(DynBlockSubject, Block, RecordPacket)> {
    (0..count)
        .map(|idx| {
            let index = (idx + 1) as u32;
            create_record((start_height.into_inner() + index).into(), prefix)
        })
        .collect()
}

pub async fn insert_records(
    db: &Arc<Db>,
    prefix: &str,
    records: &[(DynBlockSubject, Block, RecordPacket)],
) -> anyhow::Result<Vec<BlockDbItem>> {
    let mut final_records = vec![];
    for record in records {
        let packet = record.2.to_owned().with_namespace(prefix);
        let db_item: BlockDbItem = (&packet).try_into()?;
        let record = Block::insert(db.pool_ref(), &db_item).await?;
        final_records.push(record);
    }
    Ok(final_records)
}

pub async fn insert_records_with_transaction(
    tx: &mut DbTransaction,
    prefix: &str,
    records: &[(DynBlockSubject, Block, RecordPacket)],
) -> anyhow::Result<()> {
    let mut final_records = vec![];
    for record in records {
        let packet = record.2.to_owned().with_namespace(prefix);
        let db_item: BlockDbItem = (&packet).try_into()?;
        let record = Block::insert_with_transaction(tx, &db_item).await?;
        final_records.push(record);
    }
    Ok(())
}

pub fn create_random_db_name() -> String {
    format!("test_{}", rand::rng().random_range(0..1000000))
}

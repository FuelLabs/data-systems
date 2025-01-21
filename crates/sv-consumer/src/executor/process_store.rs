use std::{sync::Arc, time::Duration};

use fuel_streams_domains::MsgPayload;
use fuel_streams_store::{
    db::Db,
    record::{DbTransaction, RecordEntity, RecordPacket},
};

use super::block_stats::{ActionType, BlockStats};
use crate::{errors::ConsumerError, FuelStores};

const DB_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn handle_store_insertions(
    db: &Arc<Db>,
    fuel_stores: &Arc<FuelStores>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Store);
    let result = tokio::time::timeout(
        DB_TIMEOUT,
        process_store_packets(db, fuel_stores, packets),
    )
    .await
    .map_err(|_| ConsumerError::DatabaseTimeout)?;

    match result {
        Ok(packet_count) => Ok(stats.finish(packet_count)),
        Err(e) => Ok(stats.finish_with_error(e)),
    }
}

async fn process_store_packets(
    db: &Db,
    fuel_stores: &FuelStores,
    packets: &[RecordPacket],
) -> Result<usize, ConsumerError> {
    let mut tx = db.pool.begin().await?;
    for packet in packets {
        process_packet(fuel_stores, &mut tx, packet).await?;
    }
    tx.commit().await?;
    Ok(packets.len())
}

async fn process_packet(
    fuel_stores: &FuelStores,
    db_tx: &mut DbTransaction,
    packet: &RecordPacket,
) -> Result<(), ConsumerError> {
    let entity = packet.get_entity()?;
    match entity {
        RecordEntity::Block => {
            fuel_stores
                .blocks
                .insert_record_with_transaction(db_tx, packet)
                .await?;
        }
        RecordEntity::Transaction => {
            fuel_stores
                .transactions
                .insert_record_with_transaction(db_tx, packet)
                .await?;
        }
        RecordEntity::Input => {
            fuel_stores
                .inputs
                .insert_record_with_transaction(db_tx, packet)
                .await?;
        }
        RecordEntity::Output => {
            fuel_stores
                .outputs
                .insert_record_with_transaction(db_tx, packet)
                .await?;
        }
        RecordEntity::Receipt => {
            fuel_stores
                .receipts
                .insert_record_with_transaction(db_tx, packet)
                .await?;
        }
        RecordEntity::Utxo => {
            fuel_stores
                .utxos
                .insert_record_with_transaction(db_tx, packet)
                .await?;
        }
    }
    Ok(())
}

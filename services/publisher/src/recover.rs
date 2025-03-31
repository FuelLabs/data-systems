use std::sync::Arc;

use fuel_streams_core::types::{FuelCoreLike, Transaction};
use fuel_streams_domains::{
    infra::{db::Db, record::PacketBuilder, repository::Repository},
    predicates::{Predicate, PredicateDbItem},
    transactions::{main_tx_packet, TransactionDbItem},
    Metadata,
    MsgPayload,
};
use fuel_streams_types::BlockHeight;
use tokio::{sync::Semaphore, task::JoinSet};

const BATCH: u32 = 100;
const MAX_CONCURRENT_TASKS: usize = 32;

pub async fn recover_blob_transactions(
    db: &Arc<Db>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    initial_height: u32,
    last_block_height: &BlockHeight,
) -> anyhow::Result<()> {
    let mut iteration = initial_height;
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));

    while iteration <= **last_block_height as u32 {
        let end =
            std::cmp::min(iteration + BATCH, **last_block_height as u32 + 1);
        let mut join_set = JoinSet::new();

        for height in iteration..end {
            let db = Arc::clone(db);
            let fuel_core = Arc::clone(fuel_core);
            let block = fuel_core.get_sealed_block(height.into())?;
            let metadata = Metadata::new(&fuel_core, &block);
            let msg_payload =
                MsgPayload::new(&fuel_core, &block, &metadata).await?.arc();

            // Spawn task for transaction packets
            join_set.spawn({
                let _permit = semaphore.clone().acquire_owned().await?;
                let db = Arc::clone(&db);
                let msg_payload = msg_payload.clone();
                async move {
                    let mut db_tx = db.pool_ref().begin().await?;
                    let txs = msg_payload.transactions.clone();
                    let blob_txs = txs.iter().filter(|tx| tx.blob_id.is_some());
                    tracing::info!(
                        "Found {} blob transactions on height {}",
                        blob_txs.to_owned().count(),
                        height
                    );

                    for (tx_index, tx) in blob_txs.clone().enumerate() {
                        let packets =
                            main_tx_packet(&msg_payload, tx, tx_index);
                        let packet = packets.first();
                        if let Some(packet) = packet {
                            let db_item = TransactionDbItem::try_from(packet)?;
                            Transaction::insert(&mut *db_tx, &db_item).await?;
                        }
                    }
                    db_tx.commit().await?;
                    Ok::<(), anyhow::Error>(())
                }
            });

            // Spawn task for predicate packets
            join_set.spawn({
                let _permit = semaphore.clone().acquire_owned().await?;
                let db = Arc::clone(&db);
                let msg_payload = msg_payload.clone();
                async move {
                    let mut db_tx = db.pool_ref().begin().await?;
                    let txs = msg_payload.transactions.clone();
                    let blob_txs = txs.iter().filter(|tx| tx.blob_id.is_some());

                    for (tx_index, tx) in blob_txs.enumerate() {
                        let packets = Predicate::build_packets(&(
                            msg_payload.as_ref().clone(),
                            tx_index,
                            tx.to_owned(),
                        ));

                        tracing::info!(
                            "Found {} packets on height {}",
                            packets.len(),
                            height
                        );

                        for packet in packets {
                            let db_item = PredicateDbItem::try_from(&packet)?;
                            Predicate::insert(&mut *db_tx, &db_item).await?;
                            tracing::info!(
                                "Inserted predicate {} at tx {}",
                                db_item.predicate_address,
                                tx.id
                            );
                        }
                    }
                    db_tx.commit().await?;
                    Ok::<(), anyhow::Error>(())
                }
            });
        }

        // Wait for all tasks in the batch to complete
        while let Some(result) = join_set.join_next().await {
            result??;
        }

        iteration += BATCH;
    }

    Ok(())
}

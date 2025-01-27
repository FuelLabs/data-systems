//! TEMPORARY FILE
//! This module contains recovery logic for transactions with 'none' status.
//! It can be safely deleted once all transactions have been recovered and
//! there are no more transactions with 'none' status in the database,
//! or we can use this to create a proper failure mechanism as a service in
//! in the future.

use std::sync::Arc;

use fuel_message_broker::MessageBroker;
use fuel_streams_core::types::{BlockHeight, FuelCoreLike};
use fuel_streams_store::db::Db;
use fuel_web_utils::telemetry::Telemetry;

use crate::{error::PublishError, metrics::Metrics, publish::publish_block};

pub async fn recover_block_with_tx_status_none(
    db: &Db,
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let db = db.to_owned().arc();

    // Get total count of distinct block heights
    let count: i64 = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(DISTINCT block_height) FROM transactions WHERE tx_status = 'none'",
    )
    .fetch_one(&db.pool)
    .await?
    .0;

    tracing::info!("Found {} distinct blocks to recover", count);
    const BATCH_SIZE: i64 = 100;
    let mut offset = 0;

    while offset < count {
        tracing::info!(
            "Processing batch of blocks: {}-{} of {}",
            offset,
            offset + BATCH_SIZE.min(count - offset),
            count
        );

        // Fetch batch of distinct block heights
        let block_heights = sqlx::query_as::<_, (i64,)>(
            "SELECT DISTINCT block_height FROM transactions WHERE tx_status = 'none' \
             ORDER BY block_height LIMIT $1 OFFSET $2"
        )
        .bind(BATCH_SIZE)
        .bind(offset)
        .fetch_all(&db.pool)
        .await?;

        // Process each block height
        for (block_height_u64,) in block_heights {
            let block_height = BlockHeight::from(block_height_u64);
            let sealed_block = fuel_core.get_sealed_block(block_height);
            let sealed_block = Arc::new(sealed_block);
            tracing::info!("Recovering block #{}", block_height);
            publish_block(message_broker, fuel_core, &sealed_block, telemetry)
                .await?;

            // Delete all transactions with status 'none' for this block height
            let deleted = sqlx::query(
                "DELETE FROM transactions WHERE block_height = $1 AND tx_status = 'none'"
            )
            .bind(block_height_u64)
            .execute(&db.pool)
            .await?;
            tracing::info!(
                "Deleted {} transactions with status 'none' from block #{}",
                deleted.rows_affected(),
                block_height
            );
        }
        offset += BATCH_SIZE;
    }
    tracing::info!("Recovery process completed successfully");
    Ok(())
}

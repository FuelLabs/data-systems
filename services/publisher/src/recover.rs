use std::sync::Arc;

use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::types::FuelCoreLike;
use fuel_streams_store::db::Db;
use fuel_web_utils::telemetry::Telemetry;
use tokio::sync::Semaphore;

use crate::{metrics::Metrics, publish::publish_block};

pub async fn recover_tx_status_none(
    db: &Db,
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> anyhow::Result<()> {
    let db = db.to_owned().arc();
    let semaphore = Arc::new(Semaphore::new(10));

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

        let block_heights = sqlx::query_as::<_, (i64,)>(
            "SELECT DISTINCT block_height FROM transactions WHERE tx_status = 'none' \
             ORDER BY block_height LIMIT $1 OFFSET $2"
        )
        .bind(BATCH_SIZE)
        .bind(offset)
        .fetch_all(&db.pool)
        .await?;

        let mut join_set: tokio::task::JoinSet<anyhow::Result<()>> =
            tokio::task::JoinSet::new();
        for (block_height,) in block_heights {
            join_set.spawn({
                let db = db.clone();
                let semaphore = semaphore.clone();
                let message_broker = message_broker.clone();
                let fuel_core = fuel_core.clone();
                let telemetry = telemetry.clone();
                async move {
                    let _permit = semaphore.acquire().await.map_err(|e| {
                        tracing::error!("Failed to acquire semaphore for block #{}: {}", block_height, e);
                        anyhow::anyhow!("Semaphore error: {}", e)
                    })?;

                    let block_height_str = block_height.to_string();
                    let sealed_block = fuel_core.get_sealed_block(block_height.into()).map_err(|e| {
                        tracing::error!("Failed to get sealed block #{}: {}", block_height, e);
                        anyhow::anyhow!("Get sealed block error: {}", e)
                    })?;

                    let sealed_block = Arc::new(sealed_block);
                    tracing::info!("Recovering block #{}", block_height);

                    publish_block(&message_broker, &fuel_core, &sealed_block, &telemetry).await.map_err(|e| {
                        tracing::error!("Failed to publish block #{}: {}", block_height, e);
                        anyhow::anyhow!("Publish block error: {}", e)
                    })?;

                    sqlx::query(
                        "DELETE FROM transactions WHERE block_height = $1 AND tx_status = 'none'"
                    )
                    .bind(block_height_str)
                    .execute(&db.pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to delete transactions for block #{}: {}", block_height, e);
                        anyhow::anyhow!("Delete transactions error: {}", e)
                    })?;

                    tracing::info!(
                        "Successfully processed and deleted transactions for block #{}",
                        block_height
                    );
                    Ok(())
                }
            });
        }

        // Wait for all tasks in the batch to complete, but don't fail if individual tasks fail
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                tracing::error!("Task join error: {}", e);
            }
        }
        offset += BATCH_SIZE;
    }

    tracing::info!("Recovery process completed successfully");
    Ok(())
}

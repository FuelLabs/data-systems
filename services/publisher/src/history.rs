use std::sync::Arc;

use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::types::*;
use fuel_streams_domains::infra::Db;
use fuel_web_utils::{shutdown::ShutdownController, telemetry::Telemetry};
use tokio::{sync::Semaphore, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::{
    error::PublishError,
    gaps::{find_next_block_to_save, BlockHeightGap},
    metrics::Metrics,
    publish::publish_block,
};

pub async fn process_historical_gaps(
    from_block: BlockHeight,
    db: &Arc<Db>,
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    last_block_height: &Arc<BlockHeight>,
    shutdown: &Arc<ShutdownController>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<
    impl std::future::Future<Output = Result<(), PublishError>>,
    anyhow::Error,
> {
    let gaps = Arc::new(
        find_next_block_to_save(db, *last_block_height.clone()).await?,
    );
    tracing::info!("Found {} block gaps to process", gaps.len());
    for gap in gaps.iter() {
        tracing::info!("Gap: {} to {}", gap.start, gap.end);
    }
    Ok(process_historical_blocks(
        from_block,
        message_broker,
        fuel_core,
        last_block_height,
        &gaps,
        shutdown.token().clone(),
        telemetry,
    ))
}

fn process_historical_blocks(
    from_block: BlockHeight,
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    last_block_height: &Arc<BlockHeight>,
    gaps: &Arc<Vec<BlockHeightGap>>,
    token: CancellationToken,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> impl std::future::Future<Output = Result<(), PublishError>> {
    let message_broker = message_broker.clone();
    let fuel_core = fuel_core.clone();
    let gaps = gaps.to_vec();
    let last_block_height = *last_block_height.clone();
    let telemetry = telemetry.clone();

    async move {
        if token.is_cancelled() {
            tracing::info!("Historical block processor received shutdown signal before starting");
            return Ok(());
        }

        let Some(processed_gaps) =
            get_historical_block_range(from_block, &gaps, last_block_height)
        else {
            return Ok(());
        };

        process_blocks_with_join_set(
            processed_gaps,
            message_broker,
            fuel_core,
            telemetry,
            token,
        )
        .await;

        Ok(())
    }
}

async fn process_blocks_with_join_set(
    processed_gaps: Vec<BlockHeightGap>,
    message_broker: Arc<NatsMessageBroker>,
    fuel_core: Arc<dyn FuelCoreLike>,
    telemetry: Arc<Telemetry<Metrics>>,
    token: CancellationToken,
) {
    let semaphore = Arc::new(Semaphore::new(32));
    let mut join_set = JoinSet::new();

    // Spawn tasks for each block height
    'outer: for gap in processed_gaps {
        for height in *gap.start..=*gap.end {
            if token.is_cancelled() {
                break 'outer;
            }

            let message_broker = message_broker.clone();
            let fuel_core = fuel_core.clone();
            let telemetry = telemetry.clone();
            let semaphore = semaphore.clone();
            let token = token.clone();

            join_set.spawn(async move {
                if token.is_cancelled() {
                    return Ok(());
                }

                let _permit = semaphore.acquire().await.unwrap();
                let height = height.into();
                let sealed_block = fuel_core.get_sealed_block(height)?;
                let sealed_block = Arc::new(sealed_block);

                publish_block(
                    &message_broker,
                    &fuel_core,
                    &sealed_block,
                    &telemetry,
                    None,
                )
                .await
            });
        }
    }

    tracing::info!("Waiting for remaining tasks to complete...");

    while let Some(result) = join_set.join_next().await {
        if token.is_cancelled() {
            tracing::info!(
                "Shutdown signal received, aborting remaining tasks..."
            );
            join_set.abort_all();
            break;
        }

        match result {
            Ok(Ok(_)) => {
                tracing::debug!("Block processed successfully")
            }
            Ok(Err(e)) => {
                tracing::error!("Error processing block: {:?}", e)
            }
            Err(e) => {
                tracing::error!("Task error: {:?}", e);
            }
        }
    }

    if token.is_cancelled() {
        tracing::info!("Waiting for aborted tasks to complete...");
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                tracing::debug!("Aborted task error: {:?}", e);
            }
        }
    }
}

fn get_historical_block_range(
    from_block: BlockHeight,
    gaps: &[BlockHeightGap],
    last_block_height: BlockHeight,
) -> Option<Vec<BlockHeightGap>> {
    if gaps.is_empty() {
        return None;
    }

    let mut processed_gaps = Vec::new();
    for gap in gaps {
        let start = std::cmp::max(from_block, gap.start);
        let end = std::cmp::min(gap.end, last_block_height);

        if start <= end {
            processed_gaps.push(BlockHeightGap { start, end });
        }
    }

    if processed_gaps.is_empty() {
        tracing::info!("No historical blocks to process");
        return None;
    }

    let total_blocks: u32 = processed_gaps
        .iter()
        .map(|gap| *gap.end - *gap.start + 1)
        .sum();

    tracing::info!(
        "Processing {total_blocks} historical blocks from {} gaps",
        processed_gaps.len()
    );
    Some(processed_gaps)
}

use std::sync::Arc;

use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::types::*;
use fuel_streams_domains::infra::Db;
use fuel_web_utils::{shutdown::ShutdownController, telemetry::Telemetry};
use tokio::{
    task::JoinSet,
    time::{interval, Duration},
};
use tokio_util::sync::CancellationToken;

use crate::{
    error::PublishError,
    gaps::{find_next_block_to_save, BlockHeightGap},
    metrics::Metrics,
    publish::publish_block,
};

#[allow(clippy::too_many_arguments)]
pub async fn process_historical_gaps_periodically(
    interval_secs: u64,
    from_block: BlockHeight,
    db: &Arc<Db>,
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    last_block_height: &Arc<BlockHeight>,
    shutdown: &Arc<ShutdownController>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), anyhow::Error> {
    if interval_secs == 0 {
        tracing::info!("Historical gap processing is disabled");
        return Ok(());
    }

    // Run at the specified interval
    let mut interval = interval(Duration::from_secs(interval_secs));

    loop {
        if shutdown.token().is_cancelled() {
            tracing::info!("Historical gap processor received shutdown signal");
            break;
        }

        tracing::info!("Starting periodic historical gap processing");
        let result = process_historical_gaps(
            from_block,
            db,
            message_broker,
            fuel_core,
            last_block_height,
            shutdown,
            telemetry,
        )
        .await?;

        result.await?;

        interval.tick().await;
    }

    Ok(())
}

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

        let _ = process_blocks(
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

async fn process_blocks(
    processed_gaps: Vec<BlockHeightGap>,
    message_broker: Arc<NatsMessageBroker>,
    fuel_core: Arc<dyn FuelCoreLike>,
    telemetry: Arc<Telemetry<Metrics>>,
    token: CancellationToken,
) -> Result<(), anyhow::Error> {
    // Spawn tasks for each block height
    'outer: for gap in processed_gaps {
        tracing::info!("Processing gap: {} to {}", gap.start, gap.end);
        let heights = (*gap.start..=*gap.end).collect::<Vec<_>>();
        let heights_chunks = heights.chunks(100);

        for heights in heights_chunks {
            tracing::info!(
                "Processing chunk: {:?} - {:?}",
                heights.first(),
                heights.last()
            );

            if token.is_cancelled() {
                break 'outer;
            }

            let mut join_set: JoinSet<Result<Option<u32>, PublishError>> =
                JoinSet::new();
            for height in heights {
                tracing::info!("Processing block height: {}", height);
                let message_broker = message_broker.clone();
                let fuel_core = fuel_core.clone();
                let telemetry = telemetry.clone();
                let block_height = *height;
                let sealed_block =
                    fuel_core.get_sealed_block((*height).into())?;
                let sealed_block = Arc::new(sealed_block);
                let token = token.clone();

                join_set.spawn(async move {
                    if token.is_cancelled() {
                        return Ok(None);
                    }
                    tracing::info!("Publishing block height: {}", block_height);
                    publish_block(
                        &message_broker,
                        &fuel_core,
                        &sealed_block,
                        &telemetry,
                        None,
                    )
                    .await
                    .map(|_| Some(block_height))
                    .map_err(|e| {
                        tracing::error!(
                            "Error publishing block {}: {:?}",
                            block_height,
                            e
                        );
                        e
                    })
                });
            }
            while let Some(result) = join_set.join_next().await {
                if let Ok(Ok(Some(block_height))) = result {
                    tracing::info!(
                        "Block {} published successfully",
                        block_height
                    );
                }
            }
            tracing::info!(
                "Finished processing chunk: {:?} - {:?}",
                heights.first(),
                heights.last()
            );
        }
    }

    Ok(())
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

use std::sync::Arc;

use clap::Parser;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::types::*;
use fuel_streams_domains::infra::{Db, DbConnectionOpts};
use fuel_web_utils::{
    server::server_builder::ServerBuilder,
    shutdown::{shutdown_broker_with_timeout, ShutdownController},
    telemetry::Telemetry,
};
use sv_publisher::{
    cli::Cli,
    error::PublishError,
    gaps::{find_next_block_to_save, BlockHeightGap},
    metrics::Metrics,
    publish::publish_block,
    recover::recover_tx_pointers,
    state::ServerState,
};
use tokio::{sync::Semaphore, task::JoinSet};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = cli.fuel_core_config;
    let fuel_core: Arc<dyn FuelCoreLike> = FuelCore::new(config).await?;
    fuel_core.start().await?;

    let db = setup_db(&cli.db_url).await?;
    let message_broker = NatsMessageBroker::setup(&cli.nats_url, None).await?;
    let last_block_height = Arc::new(fuel_core.get_latest_block_height()?);
    let gaps =
        Arc::new(find_next_block_to_save(&db, *last_block_height).await?);
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();

    tracing::info!("Found {} block gaps to process", gaps.len());
    for gap in gaps.iter() {
        tracing::info!("Gap: {} to {}", gap.start, gap.end);
    }
    tracing::info!("Last block height: {}", last_block_height);

    let metrics = Metrics::new(None)?;
    let telemetry = Telemetry::new(Some(metrics)).await?;
    telemetry.start().await?;

    let server_state =
        ServerState::new(message_broker.clone(), Arc::clone(&telemetry));
    let server = ServerBuilder::build(&server_state, cli.telemetry_port);

    tokio::select! {
        result = async {
            tokio::join!(
                recover_tx_pointers(&db),
                process_historical_blocks(
                    cli.from_block.into(),
                    &message_broker,
                    &fuel_core,
                    &last_block_height,
                    &gaps,
                    shutdown.token().clone(),
                    &telemetry,
                ),
                process_live_blocks(
                    &message_broker,
                    &fuel_core,
                    shutdown.token().clone(),
                    &telemetry
                ),
                server.run()
            )
        } => {
            result.0?;
            result.1?;
            result.2?;
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received, waiting for processing to complete...");
            fuel_core.stop().await;
            shutdown_broker_with_timeout(&message_broker).await;
        }
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn setup_db(db_url: &str) -> Result<Arc<Db>, PublishError> {
    let db = Db::new(DbConnectionOpts {
        connection_str: db_url.to_string(),
        ..Default::default()
    })
    .await?;
    Ok(db)
}

async fn process_live_blocks(
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    token: CancellationToken,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let mut subscription = fuel_core.blocks_subscription();
    let process_fut = async {
        while let Ok(data) = subscription.recv().await {
            let sealed_block = Arc::new(data.sealed_block.to_owned());
            publish_block(
                message_broker,
                fuel_core,
                &sealed_block,
                telemetry,
                Some(&data),
            )
            .await?;
        }
        Ok::<_, PublishError>(())
    };

    tokio::select! {
        result = process_fut => {
            if let Err(e) = &result {
                tracing::error!("Live block processing error: {:?}", e);
            }
            result
        }
        _ = token.cancelled() => {
            tracing::info!("Shutdown signal received in live block processor");
            Ok(())
        }
    }
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

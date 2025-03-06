use std::sync::Arc;

use clap::Parser;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::types::*;
use fuel_streams_store::{
    db::{Db, DbConnectionOpts},
    store::{find_next_block_to_save, BlockHeightGap},
};
use fuel_web_utils::{
    server::api::build_and_spawn_web_server,
    shutdown::{shutdown_broker_with_timeout, ShutdownController},
    telemetry::Telemetry,
};
use futures::StreamExt;
use sv_publisher::{
    cli::Cli,
    error::PublishError,
    metrics::Metrics,
    publish::publish_block,
    state::ServerState,
};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = cli.fuel_core_config;
    let fuel_core: Arc<dyn FuelCoreLike> = FuelCore::new(config).await?;
    fuel_core.start().await?;

    let db = setup_db(&cli.db_url).await?;
    let message_broker = NatsMessageBroker::setup(&cli.nats_url, None).await?;
    let metrics = Metrics::new(None)?;
    let telemetry = Telemetry::new(Some(metrics)).await?;
    telemetry.start().await?;

    let server_state =
        ServerState::new(message_broker.clone(), Arc::clone(&telemetry));
    let server_handle =
        build_and_spawn_web_server(cli.telemetry_port, server_state).await?;

    let last_block_height = Arc::new(fuel_core.get_latest_block_height()?);
    let gaps =
        Arc::new(find_next_block_to_process(&db, *last_block_height).await?);
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();

    tracing::info!("Found {} block gaps to process", gaps.len());
    if !gaps.is_empty() {
        for gap in gaps.iter() {
            tracing::info!("Gap: {} to {}", gap.start, gap.end);
        }
    }
    tracing::info!("Last block height: {}", last_block_height);

    tokio::select! {
        result = async {
            let historical = process_historical_blocks(
                cli.from_height.into(),
                &message_broker,
                &fuel_core,
                &last_block_height,
                &gaps,
                shutdown.token().clone(),
                &telemetry,
            );

            let live = process_live_blocks(
                &message_broker,
                &fuel_core,
                shutdown.token().clone(),
                &telemetry
            );

            tokio::join!(historical, live)
        } => {
            result.0?;
            result.1?;
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received, waiting for processing to complete...");
            fuel_core.stop().await;
            tracing::info!("Stopping actix server ...");
            server_handle.stop(true).await;
            tracing::info!("Actix server stopped. Goodbye!");
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

async fn find_next_block_to_process(
    db: &Db,
    last_block_height: BlockHeight,
) -> Result<Vec<BlockHeightGap>, PublishError> {
    let gaps = find_next_block_to_save(db, last_block_height).await?;
    Ok(gaps)
}

fn get_historical_block_range(
    from_height: BlockHeight,
    gaps: &[BlockHeightGap],
    last_block_height: BlockHeight,
) -> Option<Vec<u64>> {
    if gaps.is_empty() {
        return None;
    }

    let mut heights = Vec::new();
    for gap in gaps {
        let start = std::cmp::max(from_height, gap.start);
        let end = std::cmp::min(gap.end, last_block_height);

        if start <= end {
            heights.extend((*start..=*end).map(|h| h));
        }
    }

    if heights.is_empty() {
        tracing::info!("No historical blocks to process");
        return None;
    }

    let block_count = heights.len();
    tracing::info!(
        "Processing {block_count} historical blocks from {} gaps",
        gaps.len()
    );
    Some(heights)
}

fn process_historical_blocks(
    from_height: BlockHeight,
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    last_block_height: &Arc<BlockHeight>,
    gaps: &Arc<Vec<BlockHeightGap>>,
    token: CancellationToken,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> tokio::task::JoinHandle<()> {
    let message_broker = message_broker.clone();
    let fuel_core = fuel_core.clone();
    let gaps = gaps.to_vec();

    tokio::spawn({
        let last_block_height = *last_block_height.clone();
        let telemetry = telemetry.clone();
        async move {
            let Some(heights) = get_historical_block_range(
                from_height,
                &gaps,
                last_block_height,
            ) else {
                return;
            };

            futures::stream::iter(heights)
                .map(|height| {
                    let message_broker = message_broker.clone();
                    let fuel_core = fuel_core.clone();
                    let telemetry = telemetry.clone();
                    async move {
                        let sealed_block =
                            fuel_core.get_sealed_block(height.into())?;
                        let sealed_block = Arc::new(sealed_block);
                        publish_block(
                            &message_broker,
                            &fuel_core,
                            &sealed_block,
                            &telemetry,
                        )
                        .await
                    }
                })
                .buffered(100)
                .take_until(token.cancelled())
                .for_each(|result| async move {
                    match result {
                        Ok(_) => {
                            tracing::debug!("Block processed successfully")
                        }
                        Err(e) => {
                            tracing::error!("Error processing block: {:?}", e)
                        }
                    }
                })
                .await;
        }
    })
}

async fn process_live_blocks(
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    token: CancellationToken,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let mut subscription = fuel_core.blocks_subscription();
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                tracing::info!("Shutdown signal received in live block processor");
                break;
            }
            result = subscription.recv() => {
                match result {
                    Ok(data) => {
                        let sealed_block = Arc::new(data.sealed_block.clone());
                        publish_block(
                            message_broker,
                            fuel_core,
                            &sealed_block,
                            telemetry,
                        )
                        .await?;
                    }
                    Err(_) => {
                        tracing::error!("Block subscription error");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

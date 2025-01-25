use std::sync::Arc;

use clap::Parser;
use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::types::*;
use fuel_streams_store::{
    db::{Db, DbConnectionOpts},
    record::QueryOptions,
    store::find_last_block_height,
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
    publish::{process_transactions_status_none, publish_block},
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
    let message_broker = setup_message_broker(&cli.nats_url).await?;
    let telemetry = Telemetry::new(None).await?;
    telemetry.start().await?;

    let server_state =
        ServerState::new(message_broker.clone(), Arc::clone(&telemetry));
    let server_handle =
        build_and_spawn_web_server(cli.telemetry_port, server_state).await?;

    let last_block_height = Arc::new(fuel_core.get_latest_block_height()?);
    let last_published = Arc::new(find_last_published_height(&db).await?);
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();

    tracing::info!("Last published height: {}", last_published);
    tracing::info!("Last block height: {}", last_block_height);
    tokio::select! {
        result = async {
            let historical = process_historical_blocks(
                cli.from_height.into(),
                &message_broker,
                &fuel_core,
                &last_block_height,
                &last_published,
                shutdown.token().clone(),
                &telemetry,
            );

            let recover_blocks = process_transactions_status_none(
                &db,
                &message_broker,
                &fuel_core,
                &telemetry
            );

            let live = process_live_blocks(
                &message_broker,
                &fuel_core,
                shutdown.token().clone(),
                &telemetry
            );

            tokio::join!(historical, recover_blocks, live)
        } => {
            result.0?;
            result.1?;
            result.2?;
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

async fn setup_db(db_url: &str) -> Result<Db, PublishError> {
    let db = Db::new(DbConnectionOpts {
        connection_str: db_url.to_string(),
        ..Default::default()
    })
    .await?;
    Ok(db)
}

async fn setup_message_broker(
    nats_url: &str,
) -> Result<Arc<dyn MessageBroker>, PublishError> {
    let broker = MessageBrokerClient::Nats;
    let broker = broker.start(nats_url).await?;
    broker.setup().await?;
    Ok(broker)
}

async fn find_last_published_height(
    db: &Db,
) -> Result<BlockHeight, PublishError> {
    let opts = QueryOptions::default();
    let height = find_last_block_height(db, opts).await?;
    Ok(height)
}

fn get_historical_block_range(
    from_height: BlockHeight,
    last_published_height: BlockHeight,
    last_block_height: BlockHeight,
) -> Option<Vec<u64>> {
    let last_published_height = if last_published_height > from_height {
        last_published_height
    } else {
        from_height
    };

    let last_block_height = if last_block_height > from_height {
        *last_block_height
    } else {
        tracing::error!("Last block height is less than from height");
        *last_block_height
    };

    let start_height = *last_published_height + 1;
    let end_height = last_block_height;
    if start_height > end_height {
        tracing::info!("No historical blocks to process");
        return None;
    }
    let block_count = end_height - start_height + 1;
    let heights: Vec<u64> = (start_height..=end_height).collect();
    tracing::info!(
        "Processing {block_count} historical blocks from height {start_height} to {end_height}"
    );
    Some(heights)
}

fn process_historical_blocks(
    from_height: BlockHeight,
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    last_block_height: &Arc<BlockHeight>,
    last_published_height: &Arc<BlockHeight>,
    token: CancellationToken,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> tokio::task::JoinHandle<()> {
    let message_broker = message_broker.clone();
    let fuel_core = fuel_core.clone();
    tokio::spawn({
        let last_published_height = *last_published_height.clone();
        let last_block_height = *last_block_height.clone();
        let telemetry = telemetry.clone();
        async move {
            let Some(heights) = get_historical_block_range(
                from_height,
                last_published_height,
                last_block_height,
            ) else {
                return;
            };
            futures::stream::iter(heights)
                .map(|height| {
                    let message_broker = message_broker.clone();
                    let fuel_core = fuel_core.clone();
                    let sealed_block =
                        fuel_core.get_sealed_block_by_height(height.into());
                    let sealed_block = Arc::new(sealed_block);
                    let telemetry = telemetry.clone();
                    async move {
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
    message_broker: &Arc<dyn MessageBroker>,
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

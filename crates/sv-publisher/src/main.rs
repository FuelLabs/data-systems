use std::sync::Arc;

use clap::Parser;
use fuel_core_types::blockchain::SealedBlock;
use fuel_message_broker::{
    MessageBroker,
    MessageBrokerClient,
    MessageBrokerError,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{Metadata, MsgPayload, MsgPayloadError};
use fuel_streams_store::{
    db::{Db, DbConnectionOpts},
    record::{DataEncoder, EncoderError, QueryOptions, Record},
};
use fuel_web_utils::{
    server::api::build_and_spawn_web_server,
    shutdown::{shutdown_broker_with_timeout, ShutdownController},
    telemetry::Telemetry,
};
use futures::StreamExt;
use sv_publisher::{cli::Cli, metrics::Metrics, state::ServerState};
use tokio_util::sync::CancellationToken;

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Processing was cancelled")]
    Cancelled,
    #[error(transparent)]
    Db(#[from] fuel_streams_store::db::DbError),
    #[error(transparent)]
    FuelCore(#[from] FuelCoreError),
    #[error(transparent)]
    MsgPayload(#[from] MsgPayloadError),
    #[error(transparent)]
    Encoder(#[from] EncoderError),
    #[error(transparent)]
    MessageBrokerClient(#[from] MessageBrokerError),
}

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
                cli.from_height,
                &message_broker,
                fuel_core.clone(),
                last_block_height,
                last_published,
                shutdown.token().clone(),
                Arc::clone(&telemetry),
            );

            let live = process_live_blocks(
                &message_broker,
                fuel_core.clone(),
                shutdown.token().clone(),
                Arc::clone(&telemetry)
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

async fn find_last_published_height(db: &Db) -> Result<u32, PublishError> {
    let record = Block::find_last_record(db, QueryOptions::default()).await?;
    match record {
        Some(record) => Ok(record.block_height as u32),
        None => Ok(0),
    }
}

fn get_historical_block_range(
    from_height: u32,
    last_published_height: Arc<u32>,
    last_block_height: Arc<u32>,
) -> Option<Vec<u32>> {
    let last_published_height = if *last_published_height > from_height {
        *last_published_height
    } else {
        from_height
    };

    let last_block_height = if *last_block_height > from_height {
        *last_block_height
    } else {
        tracing::error!("Last block height is less than from height");
        *last_block_height
    };

    let start_height = last_published_height + 1;
    let end_height = last_block_height;
    if start_height > end_height {
        tracing::info!("No historical blocks to process");
        return None;
    }
    let block_count = end_height - start_height + 1;
    let heights: Vec<u32> = (start_height..=end_height).collect();
    tracing::info!(
        "Processing {block_count} historical blocks from height {start_height} to {end_height}"
    );
    Some(heights)
}

fn process_historical_blocks(
    from_height: u32,
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: Arc<dyn FuelCoreLike>,
    last_block_height: Arc<u32>,
    last_published_height: Arc<u32>,
    token: CancellationToken,
    telemetry: Arc<Telemetry<Metrics>>,
) -> tokio::task::JoinHandle<()> {
    let message_broker = message_broker.clone();
    let fuel_core = fuel_core.clone();
    tokio::spawn(async move {
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
                let sealed_block = fuel_core.get_sealed_block_by_height(height);
                let sealed_block = Arc::new(sealed_block);
                let telemetry = telemetry.clone();
                async move {
                    publish_block(
                        &message_broker,
                        &fuel_core,
                        &sealed_block,
                        telemetry,
                    )
                    .await
                }
            })
            .buffered(100)
            .take_until(token.cancelled())
            .for_each(|result| async move {
                match result {
                    Ok(_) => tracing::debug!("Block processed successfully"),
                    Err(e) => {
                        tracing::error!("Error processing block: {:?}", e)
                    }
                }
            })
            .await;
    })
}

async fn process_live_blocks(
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: Arc<dyn FuelCoreLike>,
    token: CancellationToken,
    telemetry: Arc<Telemetry<Metrics>>,
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
                            &fuel_core,
                            &sealed_block,
                            telemetry.clone(),
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

async fn publish_block(
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    sealed_block: &Arc<SealedBlock>,
    telemetry: Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let metadata = Metadata::new(fuel_core, sealed_block);
    let fuel_core = Arc::clone(fuel_core);
    let payload = MsgPayload::new(fuel_core, sealed_block, &metadata)?;
    let encoded = payload.encode().await?;

    message_broker
        .publish_block(payload.message_id(), encoded.clone())
        .await?;

    if let Some(metrics) = telemetry.base_metrics() {
        metrics.update_publisher_success_metrics(
            &payload.subject(),
            encoded.len(),
        );
    }

    tracing::info!("New block submitted: {}", payload.block_height());
    Ok(())
}

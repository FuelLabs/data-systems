use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{
    context::{Publish, PublishErrorKind},
    stream::RetentionPolicy,
    Context,
};
use clap::Parser;
use fuel_core_types::{blockchain::SealedBlock, fuel_tx::Transaction};
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{NatsClient, NatsClientOpts},
    types::{Block, FuelCoreBlock},
    Stream,
};
use futures::StreamExt;
use postcard::to_allocvec;
use serde::{Deserialize, Serialize};
use sv_emitter::{
    cli::Cli,
    fuel_core_like::{FuelCore, FuelCoreLike},
    shutdown::ShutdownController,
};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = cli.fuel_core_config;
    let fuel_core: Arc<dyn FuelCoreLike> = FuelCore::new(config).await?;
    fuel_core.start().await?;

    let nats_client = setup_nats(&cli.nats_url).await?;
    let last_block_height = fuel_core.get_latest_block_height()?;
    let last_published = find_last_published_height(&nats_client).await?;
    let shutdown = Arc::new(ShutdownController::new());

    tracing::info!("Last published height: {}", last_published);
    tracing::info!("Last block height: {}", last_block_height);

    shutdown
        .clone()
        .spawn_signal_handler()
        .run_with_cancellation(|token| {
            let fuel_core = fuel_core.clone();
            async move {
                let historical = process_historical_blocks(
                    &nats_client,
                    fuel_core.clone(),
                    last_block_height,
                    last_published,
                    token.clone(),
                );

                let live = process_live_blocks(
                    &nats_client.jetstream,
                    fuel_core.clone(),
                    token,
                );

                let _ = tokio::join!(historical, live);
                Ok(())
            }
        })
        .await?;

    shutdown.on_shutdown({
        let fuel_core = fuel_core.clone();
        move || {
            tracing::info!("Stopping Fuel Core...");
            let rt = tokio::runtime::Handle::current();
            rt.block_on(fuel_core.stop());
        }
    });

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn setup_nats(nats_url: &str) -> anyhow::Result<NatsClient> {
    let opts = NatsClientOpts::admin_opts(None);
    let opts = opts.with_custom_url(nats_url.to_string());
    let nats_client = NatsClient::connect(&opts).await?;
    let stream_name = nats_client.namespace.stream_name("block_importer");
    nats_client
        .jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name,
            subjects: vec!["block_submitted.>".to_string()],
            retention: RetentionPolicy::WorkQueue,
            duplicate_window: Duration::from_secs(1),
            ..Default::default()
        })
        .await?;

    Ok(nats_client)
}

async fn find_last_published_height(
    nats_client: &NatsClient,
) -> anyhow::Result<u32> {
    let block_stream = Stream::<Block>::get_or_init(nats_client).await;
    let last_publish_height = block_stream
        .get_last_published(BlocksSubject::WILDCARD)
        .await?;
    match last_publish_height {
        Some(block) => Ok(block.height),
        None => Ok(0),
    }
}

fn get_historical_block_range(
    last_published_height: u32,
    last_block_height: u32,
) -> Option<Vec<u32>> {
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
    nats_client: &NatsClient,
    fuel_core: Arc<dyn FuelCoreLike>,
    last_block_height: u32,
    last_published_height: u32,
    token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let jetstream = nats_client.jetstream.clone();
    tokio::spawn(async move {
        let Some(heights) = get_historical_block_range(
            last_published_height,
            last_block_height,
        ) else {
            return;
        };

        futures::stream::iter(heights)
            .map(|height| {
                let jetstream = jetstream.clone();
                let fuel_core = fuel_core.clone();
                let sealed_block = fuel_core.get_sealed_block_by_height(height);
                async move {
                    match publish_block(&jetstream, &sealed_block).await {
                        Ok(_) => tracing::debug!(
                            "Published historical block {}",
                            height
                        ),
                        Err(e) => tracing::error!(
                            "Failed to publish historical block {}: {}",
                            height,
                            e
                        ),
                    }
                }
            })
            .buffer_unordered(100)
            .take_until(token.cancelled())
            .collect::<Vec<_>>()
            .await;

        tracing::info!("Historical block processing complete");
    })
}

#[derive(Error, Debug)]
pub enum LiveBlockProcessingError {
    #[error("Failed to publish block: {0}")]
    PublishError(#[from] PublishError),

    #[error("Processing was cancelled")]
    Cancelled,
}

async fn process_live_blocks(
    jetstream: &Context,
    fuel_core: Arc<dyn FuelCoreLike>,
    token: CancellationToken,
) -> Result<(), LiveBlockProcessingError> {
    let mut subscription = fuel_core.blocks_subscription();
    while let Ok(data) = subscription.recv().await {
        if token.is_cancelled() {
            break;
        }
        if let Err(e) = publish_block(jetstream, &data.sealed_block).await {
            tracing::error!("Failed to publish live block: {}", e);
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct BlockPayload {
    block: FuelCoreBlock,
    transactions: Vec<Transaction>,
}

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Failed to serialize block payload with postcard: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("Failed to publish to NATS: {0}")]
    NatsPublish(#[from] async_nats::error::Error<PublishErrorKind>),
}

async fn publish_block(
    jetstream: &Context,
    sealed_block: &SealedBlock,
) -> Result<(), PublishError> {
    let block = sealed_block.entity.clone();
    let height = *block.header().consensus().height;
    let transactions = block.transactions_vec().clone();
    let producer = sealed_block
        .consensus
        .block_producer(&block.id())
        .unwrap_or_default();

    let subject = format!("block_submitted.{}.{}", producer, height);
    let message_id = format!("block_{height}");
    let payload = to_allocvec(&BlockPayload {
        block,
        transactions,
    })
    .map_err(PublishError::Serialization)?;

    let publish = Publish::build()
        .message_id(message_id)
        .payload(payload.into());

    jetstream
        .send_publish(subject, publish)
        .await
        .map_err(PublishError::NatsPublish)?
        .await
        .map_err(PublishError::NatsPublish)?;

    tracing::info!("New block submitted: {}", height);
    Ok(())
}

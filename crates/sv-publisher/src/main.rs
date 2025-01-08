use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{
    context::{CreateStreamErrorKind, Publish, PublishErrorKind},
    stream::RetentionPolicy,
    Context,
};
use clap::Parser;
use fuel_core_types::blockchain::SealedBlock;
use fuel_streams_core::{nats::*, types::*, FuelCore, FuelCoreLike};
use fuel_streams_executors::*;
use fuel_streams_store::{
    db::{Db, DbConnectionOpts},
    record::{DataEncoder, EncoderError, Record},
};
use futures::StreamExt;
use sv_publisher::{cli::Cli, shutdown::ShutdownController};
use tokio_util::sync::CancellationToken;

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error(transparent)]
    NatsPublish(#[from] async_nats::error::Error<PublishErrorKind>),
    #[error(transparent)]
    BlockPayload(#[from] ExecutorError),
    #[error("Failed to access offchain database: {0}")]
    OffchainDatabase(String),
    #[error(transparent)]
    Db(#[from] fuel_streams_store::db::DbError),
    #[error(transparent)]
    NatsClient(#[from] NatsError),
    #[error(transparent)]
    CreateStream(#[from] async_nats::error::Error<CreateStreamErrorKind>),
    #[error(transparent)]
    Nats(#[from] async_nats::Error),
    #[error("Failed to initialize Fuel Core: {0}")]
    FuelCore(String),
    #[error(transparent)]
    Encoder(#[from] EncoderError),
    #[error("Processing was cancelled")]
    Cancelled,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = cli.fuel_core_config;
    let fuel_core: Arc<dyn FuelCoreLike> = FuelCore::new(config).await?;
    fuel_core.start().await?;

    let db = setup_db(&cli.db_url).await?;
    let nats_client = setup_nats(&cli.nats_url).await?;
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
                &nats_client,
                fuel_core.clone(),
                last_block_height,
                last_published,
                shutdown.token().clone(),
            );

            let live = process_live_blocks(
                &nats_client.jetstream,
                fuel_core.clone(),
                shutdown.token().clone(),
            );

            tokio::join!(historical, live)
        } => {
            result.0?;
            result.1?;
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received, waiting for processing to complete...");
            fuel_core.stop().await
        }
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn setup_db(db_url: &str) -> Result<Db, PublishError> {
    let db = Db::new(DbConnectionOpts {
        connection_str: db_url.to_string(),
        pool_size: Some(5),
    })
    .await?;
    Ok(db)
}

async fn setup_nats(nats_url: &str) -> Result<NatsClient, PublishError> {
    let opts = NatsClientOpts::admin_opts()
        .with_url(nats_url.to_string())
        .with_domain("CORE".to_string());
    let nats_client = NatsClient::connect(&opts).await?;
    let stream_name = nats_client.namespace.stream_name("block_importer");
    nats_client
        .jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name,
            subjects: vec!["block_submitted.>".to_string()],
            retention: RetentionPolicy::WorkQueue,
            duplicate_window: Duration::from_secs(1),
            allow_rollup: true,
            ..Default::default()
        })
        .await?;

    Ok(nats_client)
}

async fn find_last_published_height(db: &Db) -> Result<u32, PublishError> {
    let record = Block::find_last_record(db).await?;
    match record {
        Some(record) => Ok(record.order_block as u32),
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
    nats_client: &NatsClient,
    fuel_core: Arc<dyn FuelCoreLike>,
    last_block_height: Arc<u32>,
    last_published_height: Arc<u32>,
    token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let jetstream = nats_client.jetstream.clone();
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
                let jetstream = jetstream.clone();
                let fuel_core = fuel_core.clone();
                let sealed_block = fuel_core.get_sealed_block_by_height(height);
                let sealed_block = Arc::new(sealed_block);
                async move {
                    publish_block(&jetstream, &fuel_core, &sealed_block).await
                }
            })
            .buffer_unordered(100)
            .take_until(token.cancelled())
            .collect::<Vec<_>>()
            .await;
    })
}

async fn process_live_blocks(
    jetstream: &Context,
    fuel_core: Arc<dyn FuelCoreLike>,
    token: CancellationToken,
) -> Result<(), PublishError> {
    let mut subscription = fuel_core.blocks_subscription();
    while let Ok(data) = subscription.recv().await {
        if token.is_cancelled() {
            break;
        }
        let sealed_block = Arc::new(data.sealed_block.clone());
        publish_block(jetstream, &fuel_core, &sealed_block).await?;
    }
    Ok(())
}

async fn publish_block(
    jetstream: &Context,
    fuel_core: &Arc<dyn FuelCoreLike>,
    sealed_block: &Arc<SealedBlock>,
) -> Result<(), PublishError> {
    let metadata = Metadata::new(fuel_core, sealed_block);
    let fuel_core = Arc::clone(fuel_core);
    let payload = BlockPayload::new(fuel_core, sealed_block, &metadata)?;
    let publish = Publish::build()
        .message_id(payload.message_id())
        .payload(payload.encode().await?.into());

    jetstream
        .send_publish(payload.subject(), publish)
        .await
        .map_err(PublishError::NatsPublish)?
        .await
        .map_err(PublishError::NatsPublish)?;

    tracing::info!("New block submitted: {}", payload.block_height());
    Ok(())
}

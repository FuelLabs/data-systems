use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use async_nats::jetstream::{
    consumer::{
        pull::{BatchErrorKind, Config as ConsumerConfig},
        Consumer,
    },
    context::CreateStreamErrorKind,
    stream::{ConsumerErrorKind, RetentionPolicy},
};
use clap::Parser;
use displaydoc::Display as DisplayDoc;
use fuel_streams_core::prelude::*;
use fuel_streams_executors::*;
use futures::{future::try_join_all, stream::FuturesUnordered, StreamExt};
use sv_consumer::{cli::Cli, Client};
use sv_publisher::shutdown::ShutdownController;
use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time;

#[derive(thiserror::Error, Debug, DisplayDoc)]
pub enum ConsumerError {
    /// Failed to receive batch of messages from NATS: {0}
    BatchStream(#[from] async_nats::error::Error<BatchErrorKind>),
    /// Failed to create stream: {0}
    CreateStream(#[from] async_nats::error::Error<CreateStreamErrorKind>),
    /// Failed to create consumer: {0}
    CreateConsumer(#[from] async_nats::error::Error<ConsumerErrorKind>),
    /// Failed to connect to NATS client: {0}
    NatsClient(#[from] NatsError),
    /// Failed to communicate with NATS server: {0}
    Nats(#[from] async_nats::Error),
    /// Failed to deserialize block payload from message: {0}
    Deserialization(#[from] serde_json::Error),
    /// Failed to decode UTF-8: {0}
    Utf8(#[from] std::str::Utf8Error),
    /// Failed to execute executor tasks: {0}
    Executor(#[from] ExecutorError),
    /// Failed to join tasks: {0}
    JoinTasks(#[from] tokio::task::JoinError),
    /// Failed to acquire semaphore: {0}
    Semaphore(#[from] tokio::sync::AcquireError),
    /// Failed to setup storage: {0}
    Storage(#[from] fuel_streams_core::storage::StorageError),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_timer(time::LocalTime::rfc_3339())
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .init();

    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("File .env not found: {:?}", err);
    }

    let cli = Cli::parse();
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();

    tracing::info!("Consumer started. Waiting for messages...");
    tokio::select! {
        result = async {
            process_messages(&cli, shutdown.token())
                .await
        } => {
            result?;
            tracing::info!("Processing complete");
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received");
        }
    };

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn setup_storage() -> Result<Arc<S3Storage>, ConsumerError> {
    let storage_opts = S3StorageOpts::admin_opts();
    let storage = S3Storage::new(storage_opts).await?;
    Ok(Arc::new(storage))
}

async fn setup_nats(
    cli: &Cli,
) -> Result<
    (Arc<NatsClient>, Arc<NatsClient>, Consumer<ConsumerConfig>),
    ConsumerError,
> {
    let core_client = Client::Core.create(cli).await?;
    let publisher_client = Client::Publisher.create(cli).await?;
    let stream_name = publisher_client.namespace.stream_name("block_importer");
    let stream = publisher_client
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

    let consumer = stream
        .get_or_create_consumer("block_importer", ConsumerConfig {
            durable_name: Some("block_importer".to_string()),
            ack_policy: AckPolicy::Explicit,
            ..Default::default()
        })
        .await?;

    Ok((core_client, publisher_client, consumer))
}

pub static CONSUMER_MAX_THREADS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    dotenvy::var("CONSUMER_MAX_THREADS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(available_cpus)
});

async fn process_messages(
    cli: &Cli,
    token: &CancellationToken,
) -> Result<(), ConsumerError> {
    let (core_client, publisher_client, consumer) = setup_nats(cli).await?;
    let storage = setup_storage().await?;
    let (_, publisher_stream) =
        FuelStreams::setup_all(&core_client, &publisher_client, &storage).await;

    let fuel_streams: Arc<dyn FuelStreamsExt> = publisher_stream.arc();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(64));
    while !token.is_cancelled() {
        let mut messages =
            consumer.fetch().max_messages(100).messages().await?.fuse();
        let mut futs = FuturesUnordered::new();

        while let Some(msg) = messages.next().await {
            let msg = msg?;
            let fuel_streams = fuel_streams.clone();
            let semaphore = semaphore.clone();

            tracing::debug!(
                "Received message payload length: {}",
                msg.payload.len()
            );

            let future = async move {
                match BlockPayload::decode(&msg.payload).await {
                    Ok(payload) => {
                        let payload = Arc::new(payload);
                        let start_time = std::time::Instant::now();
                        let futures = Executor::<Block>::process_all(
                            payload.clone(),
                            &fuel_streams,
                            &semaphore,
                        );
                        let results = try_join_all(futures).await?;
                        let end_time = std::time::Instant::now();
                        msg.ack().await.expect("Failed to ack message");
                        Ok((results, start_time, end_time, payload))
                    }
                    Err(e) => {
                        tracing::error!("Failed to decode payload: {:?}", e);
                        tracing::debug!(
                            "Raw payload (hex): {:?}",
                            hex::encode(&msg.payload)
                        );
                        Err(e)
                    }
                }
            };
            futs.push(future);
        }
        while let Some(result) = futs.next().await {
            let (results, start_time, end_time, payload) = result?;
            log_task(results, start_time, end_time, payload);
        }
    }
    Ok(())
}

fn log_task(
    res: Vec<Result<(), ExecutorError>>,
    start_time: std::time::Instant,
    end_time: std::time::Instant,
    payload: Arc<BlockPayload>,
) {
    let height = payload.metadata().clone().block_height;
    let has_error = res.iter().any(|r| r.is_err());
    let errors = res
        .iter()
        .filter_map(|r| r.as_ref().err())
        .collect::<Vec<_>>();

    let elapsed = end_time.duration_since(start_time);
    if has_error {
        tracing::error!(
            "Block {height} published with errors in {:?}",
            elapsed
        );
        tracing::debug!("Errors: {:?}", errors);
    } else {
        tracing::info!(
            "Block {height} published successfully in {:?}",
            elapsed
        );
    }
}

use std::sync::{Arc, LazyLock};

use clap::Parser;
use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::{types::*, FuelStreams};
use fuel_streams_executors::*;
use fuel_streams_store::{
    db::{Db, DbConnectionOpts},
    record::DataEncoder,
};
use futures::{future::try_join_all, stream::FuturesUnordered, StreamExt};
use sv_consumer::cli::Cli;
use sv_publisher::shutdown::ShutdownController;
use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time;

#[derive(thiserror::Error, Debug)]
pub enum ConsumerError {
    #[error(transparent)]
    Deserialization(#[from] bincode::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Executor(#[from] ExecutorError),
    #[error(transparent)]
    JoinTasks(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Semaphore(#[from] tokio::sync::AcquireError),
    #[error(transparent)]
    Db(#[from] fuel_streams_store::db::DbError),
    #[error(transparent)]
    MessageBrokerClient(#[from] fuel_message_broker::MessageBrokerError),
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

async fn setup_db(db_url: &str) -> Result<Arc<Db>, ConsumerError> {
    let db = Db::new(DbConnectionOpts {
        connection_str: db_url.to_string(),
        pool_size: Some(5),
    })
    .await?;
    Ok(db.arc())
}

async fn setup_message_broker(
    nats_url: &str,
) -> Result<Arc<dyn MessageBroker>, ConsumerError> {
    let broker = MessageBrokerClient::Nats;
    let broker = broker.start(nats_url).await?;
    broker.setup().await?;
    Ok(broker)
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
    let db = setup_db(&cli.db_url).await?;
    let message_broker = setup_message_broker(&cli.nats_url).await?;
    let fuel_streams = FuelStreams::new(&message_broker, &db).await.arc();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(64));
    while !token.is_cancelled() {
        let mut messages =
            message_broker.receive_blocks_stream(100).await?.fuse();
        let mut futs = FuturesUnordered::new();

        while let Some(msg) = messages.next().await {
            let msg = msg?;
            let payload = msg.payload();
            let fuel_streams = fuel_streams.clone();
            let semaphore = semaphore.clone();
            tracing::debug!(
                "Received message payload length: {}",
                payload.len()
            );

            let future = async move {
                match BlockPayload::decode(&payload).await {
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
                            hex::encode(payload)
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

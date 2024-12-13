use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{
    consumer::{
        pull::{BatchErrorKind, Config as ConsumerConfig},
        Consumer,
    },
    context::CreateStreamErrorKind,
    stream::{ConsumerErrorKind, RetentionPolicy},
};
use clap::Parser;
use fuel_streams_core::prelude::*;
use fuel_streams_executors::*;
use futures::StreamExt;
use sv_consumer::{cli::Cli, Client};
use sv_emitter::shutdown::ShutdownController;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::fmt::time;

#[derive(thiserror::Error, Debug)]
pub enum ConsumerError {
    #[error("Failed to receive batch of messages from NATS: {0}")]
    BatchStream(#[from] async_nats::error::Error<BatchErrorKind>),

    #[error("Failed to create stream: {0}")]
    CreateStream(#[from] async_nats::error::Error<CreateStreamErrorKind>),

    #[error("Failed to create consumer: {0}")]
    CreateConsumer(#[from] async_nats::error::Error<ConsumerErrorKind>),

    #[error("Failed to connect to NATS client: {0}")]
    NatsClient(#[from] NatsError),

    #[error("Failed to communicate with NATS server: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Failed to deserialize block payload from message: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Failed to publish block to stream: {0}")]
    Publish(#[from] ExecutorError),

    #[error("Failed to decode UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("sv_consumer=trace,fuel_streams_executors=trace")
        .with_timer(time::LocalTime::rfc_3339())
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .init();

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

async fn setup_nats(
    cli: &Cli,
) -> Result<
    (Arc<NatsClient>, Arc<NatsClient>, Consumer<ConsumerConfig>),
    ConsumerError,
> {
    let core_client = Client::Core.create(cli).await?;
    let publisher_client = Client::Publisher.create(cli).await?;
    let stream_name = core_client.namespace.stream_name("block_importer");
    let stream = core_client
        .jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name,
            subjects: vec!["block_submitted.>".to_string()],
            retention: RetentionPolicy::WorkQueue,
            duplicate_window: Duration::from_secs(1),
            ..Default::default()
        })
        .await?;

    let consumer = stream
        .get_or_create_consumer("block_importer", ConsumerConfig {
            ack_policy: AckPolicy::Explicit,
            ..Default::default()
        })
        .await?;

    Ok((core_client, publisher_client, consumer))
}

async fn process_messages(
    cli: &Cli,
    token: &CancellationToken,
) -> Result<(), ConsumerError> {
    let (_, publisher_client, consumer) = setup_nats(cli).await?;
    let fuel_streams = FuelStreams::new(&publisher_client, None).await;
    let fuel_streams: Arc<dyn FuelStreamsExt> = fuel_streams.arc();

    while !token.is_cancelled() {
        let messages = consumer.fetch().max_messages(100).messages().await?;
        let fuel_streams = fuel_streams.clone();
        tokio::pin!(messages);
        while let Some(msg) = messages.next().await {
            let msg = msg?;
            let msg_str = std::str::from_utf8(&msg.payload)?;
            let payload = Arc::new(BlockPayload::decode(msg_str)?);
            Executor::<Block>::process_all(payload, &fuel_streams).await?;
            msg.ack().await?;
        }
    }
    Ok(())
}

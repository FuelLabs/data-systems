use std::{str::FromStr, sync::Arc, time::Duration};

use async_nats::jetstream::{
    consumer::{
        pull::{BatchErrorKind, Config as ConsumerConfig},
        Consumer,
    },
    stream::RetentionPolicy,
};
use clap::Parser;
use fuel_core_client::client::FuelClient;
use fuel_streams_core::prelude::*;
use futures::{future::try_join_all, stream::FuturesUnordered, StreamExt};
use sv_consumer::{
    cli::Cli,
    fuel_streams::FuelStreams,
    payloads,
    publisher::PublishOpts,
};
use sv_emitter::{shutdown::ShutdownController, BlockPayload};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::fmt::time;

#[derive(thiserror::Error, Debug)]
pub enum ConsumerError {
    #[error("Failed to receive batch of messages from NATS: {0}")]
    Batch(#[from] async_nats::error::Error<BatchErrorKind>),

    #[error("Failed to communicate with NATS server: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Failed to deserialize block payload from message: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Failed to publish block to stream: {0}")]
    Publish(#[from] fuel_streams_core::StreamError),

    #[error("Failed to decode UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Failed to connect to Fuel Core client: {0}")]
    ClientConnect(#[from] anyhow::Error),

    #[error("Failed to join all tasks: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("sv_consumer=trace")
        .with_timer(time::LocalTime::rfc_3339())
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .init();

    let cli = Cli::parse();
    let (nats_client, consumer) = setup_nats(&cli).await?;
    let shutdown = Arc::new(ShutdownController::new());
    let fuel_client =
        Arc::new(FuelClient::from_str("0.0.0.0:4000/v1/graphql")?);
    shutdown.clone().spawn_signal_handler();

    tracing::info!("Consumer started. Waiting for messages...");
    tokio::select! {
        result = async {
            process_messages(&fuel_client, &nats_client, consumer, shutdown.token()).await
        } => {
            result?;
            tracing::info!("Processing complete");
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received");
            nats_client.nats_client.flush().await?
        }
    };

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn process_messages(
    fuel_client: &Arc<FuelClient>,
    nats_client: &Arc<NatsClient>,
    consumer: Consumer<ConsumerConfig>,
    token: &CancellationToken,
) -> Result<(), ConsumerError> {
    let fuel_streams = FuelStreams::new(nats_client).await;
    let block_stream = Arc::new(fuel_streams.clone().blocks);

    while !token.is_cancelled() {
        let messages = consumer.fetch().max_messages(100).messages().await?;
        let fuel_client = Arc::clone(fuel_client);
        let fuel_streams = fuel_streams.clone();
        tokio::pin!(messages);
        while let Some(msg) = messages.next().await {
            let start_time = std::time::Instant::now();
            let msg = msg?;
            let msg_str = std::str::from_utf8(&msg.payload)?;
            let payload = BlockPayload::decode(msg_str)?;
            let opts: PublishOpts = payload.opts.clone().into();
            let opts = Arc::new(opts);
            let txs = payload.transactions.clone();
            let publish_tasks = payloads::transactions::publish_all_tasks(
                &txs,
                &fuel_streams,
                &opts,
                &fuel_client,
            )
            .await?
            .into_iter()
            .chain(std::iter::once(payloads::blocks::publish_task(
                &payload.block,
                block_stream.clone(),
                &opts,
                payload.tx_ids(),
            )))
            .collect::<FuturesUnordered<_>>();
            try_join_all(publish_tasks).await?;

            let elapsed = start_time.elapsed();
            let height = payload.opts.block_height;
            tracing::info!(
                "Published streams for BlockHeight: {height} in {:?}",
                elapsed
            );

            msg.ack().await?;
        }
    }
    Ok(())
}

async fn setup_nats(
    cli: &Cli,
) -> anyhow::Result<(Arc<NatsClient>, Consumer<ConsumerConfig>)> {
    let nats_url = &cli.nats_url;
    let service_name = &cli.service_name;
    let opts = NatsClientOpts::admin_opts(None);
    let opts = opts.with_custom_url(nats_url.to_string());
    let nats_client = Arc::new(NatsClient::connect(&opts).await?);
    let stream_name = nats_client.namespace.stream_name("block_importer");
    let stream = nats_client
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
        .get_or_create_consumer(service_name, ConsumerConfig {
            durable_name: Some(service_name.to_string()),
            filter_subject: "block_submitted.>".to_string(),
            ..Default::default()
        })
        .await?;

    Ok((nats_client, consumer))
}

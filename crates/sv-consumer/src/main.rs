use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{
    consumer::{
        pull::{BatchErrorKind, Config as ConsumerConfig},
        Consumer,
    },
    stream::RetentionPolicy,
};
use clap::Parser;
use fuel_core_types::fuel_tx::Transaction;
use fuel_streams_core::{
    nats::{NatsClient, NatsClientOpts},
    types::FuelCoreBlock,
};
use futures::StreamExt;
use postcard::from_bytes;
use serde::{Deserialize, Serialize};
use sv_consumer::cli::Cli;
use sv_emitter::shutdown::{ShutdownController, ShutdownError};
use tokio_util::sync::CancellationToken;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct BlockPayload {
    block: FuelCoreBlock,
    transactions: Vec<Transaction>,
}

#[derive(thiserror::Error, Debug)]
pub enum ConsumerError {
    #[error("Failed to receive message: {0}")]
    Batch(#[from] async_nats::error::Error<BatchErrorKind>),

    #[error("Failed to receive message: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Failed to deserialize message: {0}")]
    Deserialization(#[from] postcard::Error),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (nats_client, consumer) = setup_nats(&cli).await?;
    let shutdown = Arc::new(ShutdownController::new());
    tracing::info!("Consumer started. Waiting for messages...");

    shutdown
        .clone()
        .spawn_signal_handler()
        .run_with_cancellation(|token| async move {
            process_messages(consumer, token)
                .await
                .map_err(|e| ShutdownError::Cancelled(Box::new(e)))
        })
        .await?;

    shutdown.on_shutdown({
        let nats_client = nats_client.clone();
        move || {
            tracing::info!("Cleaning up NATS connection...");
            let rt = tokio::runtime::Handle::current();
            let _ = rt.block_on(nats_client.nats_client.flush());
        }
    });

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn process_messages(
    consumer: Consumer<ConsumerConfig>,
    token: CancellationToken,
) -> Result<(), ConsumerError> {
    while !token.is_cancelled() {
        let messages = consumer
            .fetch()
            .max_messages(100)
            .messages()
            .await?
            .take_until(token.cancelled());

        tokio::pin!(messages);
        while let Some(msg) = messages.next().await {
            let msg = msg?;
            let payload = from_bytes::<BlockPayload>(&msg.payload)?;
            dbg!(&payload);
            msg.ack().await?;
        }
    }

    tracing::info!("Message processing stopped");
    Ok(())
}

async fn setup_nats(
    cli: &Cli,
) -> anyhow::Result<(NatsClient, Consumer<ConsumerConfig>)> {
    let nats_url = &cli.nats_url;
    let service_name = &cli.service_name;
    let opts = NatsClientOpts::admin_opts(None);
    let opts = opts.with_custom_url(nats_url.to_string());
    let nats_client = NatsClient::connect(&opts).await?;
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

use std::sync::Arc;

use clap::Parser;
use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::FuelStreams;
use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::{shutdown::ShutdownController, telemetry::Telemetry};
use sv_consumer::{cli::Cli, errors::ConsumerError, BlockExecutor, Server};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time;

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

    // Initialize shared resources
    let db = setup_db(&cli.db_url).await?;
    let message_broker = setup_message_broker(&cli.nats_url).await?;
    let telemetry = Telemetry::new(None).await?;
    telemetry.start().await?;
    let fuel_streams = FuelStreams::new(&message_broker, &db).await.arc();
    let block_executor = BlockExecutor::new(
        db,
        &message_broker,
        &fuel_streams,
        Arc::clone(&telemetry),
    );
    let server = Server::new(cli.port, message_broker, Arc::clone(&telemetry));

    tracing::info!("Consumer started. Waiting for messages...");
    tokio::select! {
        result = async {
            tokio::join!(
                block_executor.start(shutdown.token()),
                server.start()
            )
        } => {
            result.0?;
            result.1?;
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
        ..Default::default()
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

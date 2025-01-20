use std::sync::Arc;

use clap::Parser;
use fuel_message_broker::{MessageBroker, MessageBrokerClient};
use fuel_streams_core::FuelStreams;
use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::{
    server::api::build_and_spawn_web_server,
    shutdown::ShutdownController,
    telemetry::Telemetry,
};
use sv_consumer::{
    cli::Cli,
    errors::ConsumerError,
    process_messages_from_broker,
    state::ServerState,
    FuelStores,
};
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
    let fuel_streams = FuelStreams::new(&message_broker, &db).await.arc();
    let fuel_stores = FuelStores::new(&db).arc();

    tracing::info!("Consumer started. Waiting for messages...");
    tokio::select! {
        result = async {
            tokio::join!(
                process_messages_from_broker(
                    &db,
                    shutdown.token(),
                    &message_broker,
                    &fuel_streams,
                    &fuel_stores
                ),
                start_web_server(
                    cli.port,
                    message_broker.clone(),
                    fuel_streams.clone()
                )
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
        pool_size: Some(5),
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

async fn start_web_server(
    port: u16,
    message_broker: Arc<dyn MessageBroker>,
    fuel_streams: Arc<FuelStreams>,
) -> Result<(), ConsumerError> {
    let telemetry = Telemetry::new(None)
        .await
        .map_err(|_| ConsumerError::TelemetryStart)?;

    telemetry
        .start()
        .await
        .map_err(|_| ConsumerError::TelemetryStart)?;

    let server_state =
        ServerState::new(message_broker, Arc::clone(&telemetry), fuel_streams);

    build_and_spawn_web_server(port, server_state)
        .await
        .map_err(|_| ConsumerError::WebServerStart)?;
    Ok(())
}

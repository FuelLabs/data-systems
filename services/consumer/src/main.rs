use std::sync::Arc;

use clap::Parser;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::FuelStreams;
use fuel_streams_domains::infra::{Db, DbConnectionOpts};
use fuel_web_utils::{
    server::server_builder::ServerBuilder,
    shutdown::ShutdownController,
    telemetry::Telemetry,
    tracing::init_tracing,
};
use sv_consumer::{
    cli::Cli,
    errors::ConsumerError,
    metrics::Metrics,
    server::ServerState,
    BlockExecutor,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;

    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("File .env not found: {:?}", err);
    }

    let cli = Cli::parse();
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();
    // Initialize shared resources
    let db_pool_size = cli.db_pool_size.unwrap_or(cli.concurrent_tasks);
    let db = setup_db(&cli.db_url, db_pool_size as u32).await?;
    // 2 minutes before returning to the message broker
    let opts = fuel_message_broker::NatsOpts::new(cli.nats_url.clone())
        .with_ack_wait(120);
    let message_broker = NatsMessageBroker::setup_with_opts(&opts).await?;
    let metrics = Metrics::new(None)?;
    let telemetry = Telemetry::new(Some(metrics)).await?;
    telemetry.start().await?;

    let fuel_streams = FuelStreams::new(&message_broker, &db).await.arc();
    let server_state =
        ServerState::new(message_broker.clone(), Arc::clone(&telemetry));
    let server = ServerBuilder::build(&server_state, cli.port);
    tracing::info!("Consumer started. Waiting for messages...");

    let block_executor = BlockExecutor::new(
        db,
        &message_broker,
        &fuel_streams,
        Arc::clone(&telemetry),
        cli.concurrent_tasks,
    );

    tokio::select! {
        result = async {
            tokio::join!(
                block_executor.start(shutdown.token()),
                server.run()
            )
        } => {
            result.0?;
            result.1?;
            tracing::info!("Processing complete");
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received");
        }
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn setup_db(
    db_url: &str,
    concurrent_tasks: u32,
) -> Result<Arc<Db>, ConsumerError> {
    let db = Db::new(DbConnectionOpts {
        connection_str: db_url.to_string(),
        pool_size: Some(concurrent_tasks),
        ..Default::default()
    })
    .await?;
    Ok(db)
}

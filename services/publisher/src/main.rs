use std::sync::Arc;

use clap::Parser;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_core::types::*;
use fuel_streams_domains::infra::{Db, DbConnectionOpts};
use fuel_web_utils::{
    server::server_builder::ServerBuilder,
    shutdown::{shutdown_broker_with_timeout, ShutdownController},
    telemetry::Telemetry,
};
use sv_publisher::{
    cli::Cli,
    error::PublishError,
    history::process_historical_gaps,
    metrics::Metrics,
    publish::publish_block,
    recover::recover_tx_pointers,
    state::ServerState,
};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = cli.fuel_core_config;
    let fuel_core: Arc<dyn FuelCoreLike> = FuelCore::new(config).await?;
    fuel_core.start().await?;

    let db = setup_db(&cli.db_url).await?;
    let message_broker = NatsMessageBroker::setup(&cli.nats_url, None).await?;
    let last_block_height = Arc::new(fuel_core.get_latest_block_height()?);
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();

    tracing::info!("Last block height: {}", last_block_height);

    let metrics = Metrics::new(None)?;
    let telemetry = Telemetry::new(Some(metrics)).await?;
    telemetry.start().await?;

    let server_state =
        ServerState::new(message_broker.clone(), Arc::clone(&telemetry));
    let server = ServerBuilder::build(&server_state, cli.telemetry_port);
    let historical_gaps = process_historical_gaps(
        cli.from_block.into(),
        &db,
        &message_broker,
        &fuel_core,
        &last_block_height,
        &shutdown,
        &telemetry,
    )
    .await?;

    tokio::select! {
        result = async {
            tokio::join!(
                recover_tx_pointers(&db),
                historical_gaps,
                process_live_blocks(
                    &message_broker,
                    &fuel_core,
                    shutdown.token().clone(),
                    &telemetry
                ),
                server.run()
            )
        } => {
            result.0?;
            result.1?;
            result.2?;
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received, waiting for processing to complete...");
            fuel_core.stop().await;
            shutdown_broker_with_timeout(&message_broker).await;
        }
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn setup_db(db_url: &str) -> Result<Arc<Db>, PublishError> {
    let db = Db::new(DbConnectionOpts {
        connection_str: db_url.to_string(),
        ..Default::default()
    })
    .await?;
    Ok(db)
}

async fn process_live_blocks(
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    token: CancellationToken,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let mut subscription = fuel_core.blocks_subscription();
    let process_fut = async {
        while let Ok(data) = subscription.recv().await {
            let sealed_block = Arc::new(data.sealed_block.to_owned());
            publish_block(
                message_broker,
                fuel_core,
                &sealed_block,
                telemetry,
                Some(&data),
            )
            .await?;
        }
        Ok::<_, PublishError>(())
    };

    tokio::select! {
        result = process_fut => {
            if let Err(e) = &result {
                tracing::error!("Live block processing error: {:?}", e);
            }
            result
        }
        _ = token.cancelled() => {
            tracing::info!("Shutdown signal received in live block processor");
            Ok(())
        }
    }
}

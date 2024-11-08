//! This binary subscribes to events emitted from a Fuel client or node
//! to publish streams that can consumed via the `fuel-streams` SDK.
use std::{net::ToSocketAddrs, sync::Arc, time::Duration};

use clap::Parser;
use fuel_core_bin::{cli::run as fuel_core_cli, FuelService};
use fuel_streams_publisher::{
    cli::Cli,
    metrics::PublisherMetrics,
    server::create_web_server,
    state::SharedState,
    system::System,
    PUBLISHER_MAX_THREADS,
};
use parking_lot::RwLock;
use tokio::sync::mpsc;

#[derive(Debug)]
enum RuntimeMessage {
    FuelCoreStarted,
    FuelCoreError(String),
    PublisherStopped,
}

async fn setup_fuel_core(
    config: &fuel_core_cli::Command,
) -> anyhow::Result<Arc<FuelService>> {
    let (tx, mut rx) = mpsc::channel::<RuntimeMessage>(100);
    let service = fuel_core_cli::get_service(config.to_owned()).await?;
    let fuel_core = Arc::new(service);
    let fuel_core_clone = Arc::clone(&fuel_core);

    tokio::spawn(async move {
        let result = fuel_core_clone.start_and_await().await;
        let message = match result {
            Ok(_) => RuntimeMessage::FuelCoreStarted,
            Err(e) => RuntimeMessage::FuelCoreError(e.to_string()),
        };

        if let Err(e) = tx.send(message).await {
            tracing::error!("Failed to send runtime message: {}", e);
        }
    });

    // Wait for fuel core to start
    match rx.recv().await.ok_or_else(|| {
        anyhow::anyhow!("Fuel core communication channel closed")
    })? {
        RuntimeMessage::FuelCoreStarted => {
            tracing::info!("Fuel core started successfully");
            Ok(fuel_core)
        }
        RuntimeMessage::FuelCoreError(err) => {
            Err(anyhow::anyhow!("Fuel core failed to start: {}", err))
        }
        msg => Err(anyhow::anyhow!(
            "Unexpected message from fuel core: {:?}",
            msg
        )),
    }
}

async fn setup_publisher(
    cli: &Cli,
    fuel_core: Arc<FuelService>,
) -> anyhow::Result<(SharedState, mpsc::Receiver<RuntimeMessage>)> {
    let system = Arc::new(RwLock::new(System::new().await));
    let monitoring_system = Arc::clone(&system);
    tokio::spawn(async move {
        System::monitor(&monitoring_system, Duration::from_secs(2)).await;
    });

    let state = SharedState::new(
        Arc::clone(&fuel_core),
        &cli.nats_url,
        Arc::new(PublisherMetrics::new(None)?),
        system,
    )
    .await?;

    let (tx, rx) = mpsc::channel::<RuntimeMessage>(100);
    let cli = cli.clone();

    tokio::spawn({
        let state = state.clone();
        async move {
            let publisher = fuel_streams_publisher::Publisher::new(
                state.fuel_service.clone(),
                &cli.nats_url,
                cli.use_elastic_logging,
                state.metrics.clone(),
                state.streams.clone(),
            )
            .await?;

            match publisher.run().await {
                Ok(_) => tracing::info!("Publisher completed successfully"),
                Err(err) => tracing::error!("Publisher error: {:?}", err),
            }

            if let Err(e) = tx.send(RuntimeMessage::PublisherStopped).await {
                tracing::error!("Failed to send shutdown signal: {}", e);
            }

            Ok::<(), anyhow::Error>(())
        }
    });

    Ok((state, rx))
}

async fn setup_server(
    cli: &Cli,
    state: SharedState,
) -> anyhow::Result<actix_web::dev::ServerHandle> {
    let server_addr = cli
        .server_addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing server address"))?;

    let server = create_web_server(state, server_addr)?;
    let server_handle = server.handle();
    tokio::spawn(async move {
        if let Err(err) = server.await {
            tracing::error!("Actix Web server error: {:?}", err);
        }
    });

    Ok(server_handle)
}

fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();
    let cli = Cli::parse();
    let publisher_threads = *PUBLISHER_MAX_THREADS;

    println!("Publisher threads: {}", publisher_threads);

    let fuel_core_runtime = tokio::runtime::Runtime::new()?;
    let fuel_core = fuel_core_runtime
        .block_on(async { setup_fuel_core(&cli.fuel_core_config).await })?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(publisher_threads)
        .max_blocking_threads(publisher_threads * 2)
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let (state, mut publisher_rx) =
            setup_publisher(&cli, fuel_core).await?;
        let server_handle = setup_server(&cli, state).await?;

        tracing::info!("Publisher started.");

        // Wait for shutdown
        match publisher_rx.recv().await.ok_or_else(|| {
            anyhow::anyhow!("Publisher channel closed unexpectedly")
        })? {
            RuntimeMessage::PublisherStopped => {
                tracing::info!("Publisher stopped");
            }
            msg => {
                tracing::error!(
                    "Unexpected message during shutdown: {:?}",
                    msg
                );
                return Err(anyhow::anyhow!(
                    "Unexpected shutdown message: {:?}",
                    msg
                ));
            }
        }

        // Cleanup
        tracing::info!("Stopping actix server ...");
        server_handle.stop(true).await;
        tracing::info!("Actix server stopped. Goodbye!");

        Ok(())
    })?;

    drop(runtime);
    drop(fuel_core_runtime);

    Ok(())
}

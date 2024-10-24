//! This binary subscribes to events emitted from a Fuel client or node
//! to publish streams that can consumed via the `fuel-streams` SDK.
use std::{net::ToSocketAddrs, sync::Arc, time::Duration};

use clap::Parser;
use fuel_streams_publisher::{
    cli::Cli,
    metrics::PublisherMetrics,
    server::create_web_server,
    state::SharedState,
    system::System,
};
use parking_lot::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();

    let cli = Cli::parse();

    // create the fuel core service
    let fuel_core =
        fuel_core_bin::cli::run::get_service(cli.fuel_core_config).await?;
    let fuel_core = Arc::new(fuel_core);

    // start the fuel core in the background
    fuel_core
        .start_and_await()
        .await
        .expect("Fuel core service startup failed");

    // spawn a system monitoring service
    let system = Arc::new(RwLock::new(System::new().await));
    let monitoring_system = Arc::clone(&system);
    tokio::spawn(async move {
        System::monitor(&monitoring_system, Duration::from_secs(2)).await;
    });

    // create a common shared state between actix and publisher
    let state = SharedState::new(
        Arc::clone(&fuel_core),
        &cli.nats_url,
        Arc::new(PublisherMetrics::new(None)?),
        system,
    )
    .await?;

    let publisher = fuel_streams_publisher::Publisher::new(
        state.fuel_service.clone(),
        &cli.nats_url,
        cli.use_elastic_logging,
        state.metrics.clone(),
        state.streams.clone(),
    )
    .await?;

    // create the actix webserver
    let server_addr = cli
        .server_addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing server address"))?;

    let server = create_web_server(state, server_addr)?;

    // get server handle
    let server_handle = server.handle();

    // spawn the server in the background
    tokio::spawn(async move {
        if let Err(err) = server.await {
            tracing::error!("Actix Web server error: {:?}", err);
        }
    });
    tracing::info!("Publisher started.");

    // run publisher until shutdown signal intercepted
    if let Err(err) = publisher.run().await {
        tracing::error!("Publisher encountered an error: {:?}", err);
    }
    tracing::info!("Publisher stopped");

    // Await the Actix server shutdown
    tracing::info!("Stopping actix server ...");
    server_handle.stop(true).await;

    tracing::info!("Actix server stopped. Goodbye!");

    Ok(())
}

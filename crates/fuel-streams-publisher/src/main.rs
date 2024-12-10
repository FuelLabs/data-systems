use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use clap::Parser;
use fuel_streams_publisher::{
    cli::Cli,
    publisher::shutdown::ShutdownController,
    server::{http::create_web_server, state::ServerState},
    telemetry::Telemetry,
    FuelCore,
    FuelCoreLike,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let historical = cli.clone().historical;

    let fuel_core: Arc<dyn FuelCoreLike> =
        FuelCore::new(cli.fuel_core_config).await?;
    fuel_core.start().await?;

    let telemetry = Telemetry::new().await?;
    telemetry.start().await?;

    let publisher = fuel_streams_publisher::Publisher::new(
        Arc::clone(&fuel_core),
        cli.nats_url,
        telemetry.clone(),
    )
    .await?;

    let state = ServerState::new(publisher.clone()).await;
    // create the actix webserver
    let server_addr = std::net::SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::UNSPECIFIED,
        cli.telemetry_port,
    ));
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

    let shutdown_controller = ShutdownController::new().arc();
    let shutdown_token = shutdown_controller.get_token();
    ShutdownController::spawn_signal_listener(shutdown_controller);

    // run publisher until shutdown signal intercepted
    if let Err(err) = publisher.run(shutdown_token, historical).await {
        tracing::error!("Publisher encountered an error: {:?}", err);
    }
    tracing::info!("Publisher stopped");

    // Await the Actix server shutdown
    tracing::info!("Stopping actix server ...");
    server_handle.stop(true).await;
    tracing::info!("Actix server stopped. Goodbye!");

    Ok(())
}

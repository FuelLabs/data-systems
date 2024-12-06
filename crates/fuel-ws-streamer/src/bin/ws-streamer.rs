use anyhow::Context as _;
use clap::Parser;
use fuel_ws_streamer::{
    cli::Cli,
    config::Config,
    server::{api::create_api, context::Context, state::ServerState},
};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // load envs
    dotenvy::dotenv().context("Failed to env values")?;

    // read cli args
    let cli = Cli::parse();

    // load config
    let config = match cli.config_path {
        Some(path) => {
            tracing::info!("Using config file: {}", path);
            Config::from_path(path)
                .await
                .context("Failed to load toml config")?
        }
        None => {
            tracing::info!("Using envs to load config");
            Config::from_envs().context("Failed to load toml config")?
        }
    };

    // init context
    let context = Context::new(&config).await?;

    // init server shared state
    let state = ServerState::new(context).await;

    // create the actix webserver
    let server = create_api(&config, state)?;

    // get server handle
    let server_handle = server.handle();

    // spawn the server in the background
    tokio::spawn(async move {
        if let Err(err) = server.await {
            tracing::error!("Actix Web server error: {:?}", err);
        }
    });

    // Await the Actix server shutdown
    tracing::info!("Stopping actix server ...");
    server_handle.stop(true).await;
    tracing::info!("Actix server stopped. Goodbye!");

    Ok(())
}

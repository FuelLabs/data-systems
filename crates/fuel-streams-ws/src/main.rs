use anyhow::Context as _;
use clap::Parser;
use fuel_streams_ws::{
    cli::Cli,
    config::Config,
    server::{api::create_api, context::Context, state::ServerState},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
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
    let jh = tokio::spawn(async move {
        tracing::info!("Starting actix server ...");
        if let Err(err) = server.await {
            tracing::error!("Actix Web server error: {:?}", err);
        }
    });

    let _ = tokio::join!(jh);

    // Await the Actix server shutdown
    tracing::info!("Stopping actix server ...");
    server_handle.stop(true).await;
    tracing::info!("Actix server stopped. Goodbye!");

    Ok(())
}

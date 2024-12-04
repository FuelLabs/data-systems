use anyhow::Context as _;
use clap::Parser;
use fuel_ws_streamer::{
    cli::Cli,
    config::Config,
    server::{context::Context, http::create_web_server, state::ServerState},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load envs
    dotenvy::dotenv().context("Failed to env values")?;

    // read cli args
    let cli = Cli::parse();

    // init config
    let mut config = Config::new(cli.config_path)
        .await
        .context("Failed to load toml config")?;

    // update config using envs
    config.from_envs();

    // init context
    let context = Context::new(&config).await?;

    // init server shared state
    let state = ServerState::new(context).await;

    // create the actix webserver
    let server = create_web_server(&config, state)?;

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

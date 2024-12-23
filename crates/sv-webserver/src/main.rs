use sv_webserver::{
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

    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("File .env not found: {:?}", err);
    }

    let config = Config::load()?;
    let context = Context::new(&config).await?;
    let state = ServerState::new(context).await;
    let server = create_api(&config, state)?;
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

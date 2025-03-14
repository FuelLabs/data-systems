use fuel_web_utils::server::server_builder::ServerBuilder;
use sv_webserver::{
    config::Config,
    server::{routes::create_routes, state::ServerState},
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
    let state = ServerState::new(&config).await?;
    let router = create_routes(&state);
    ServerBuilder::build(&state, config.api.port)
        .with_router(router)
        .run()
        .await?;
    Ok(())
}

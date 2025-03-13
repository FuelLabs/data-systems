use fuel_web_utils::server::api::ApiServerBuilder;
use sv_api::{
    config::Config,
    server::{handlers, state::ServerState},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[actix_web::main]
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
    let server_state = ServerState::new(&config).await?;
    let server = ApiServerBuilder::new(config.api.port, server_state.clone())
        .with_dynamic_routes(handlers::create_services(server_state))
        .build()?;

    server.await?;
    Ok(())
}

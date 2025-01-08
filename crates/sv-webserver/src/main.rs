use fuel_web_utils::server::api::{spawn_web_server, ApiServerBuilder};
use sv_webserver::{
    config::Config,
    server::{state::ServerState, svc::svc_handlers},
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
    let server_state = ServerState::new(&config).await?;
    let server = ApiServerBuilder::new(config.api.port, server_state.clone())
        .with_dynamic_routes(svc_handlers(server_state))
        .build()?;
    let server_handle = server.handle();
    let server_task = spawn_web_server(server).await;
    let _ = tokio::join!(server_task);

    // Await the Actix server shutdown
    tracing::info!("Stopping actix server ...");
    server_handle.stop(true).await;
    tracing::info!("Actix server stopped. Goodbye!");

    Ok(())
}

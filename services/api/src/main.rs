use fuel_web_utils::{
    server::server_builder::ServerBuilder,
    tracing::init_tracing,
};
use sv_api::{
    config::Config,
    server::{db_metrics, routes::create_routes, state::ServerState},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;

    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("File .env not found: {:?}", err);
    }

    let config = Config::load()?;
    let state = ServerState::new(&config).await?;
    let router = create_routes(&state);
    db_metrics::spawn_block_height_monitor(&state).await;

    ServerBuilder::build(&state, config.api.port)
        .with_router(router)
        .run()
        .await?;

    Ok(())
}

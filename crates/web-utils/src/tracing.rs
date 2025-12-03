use tracing::Level;
use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_tracing() -> anyhow::Result<()> {
    let mut env_filter = EnvFilter::from_default_env()
        .add_directive("fuel_web_utils::server=trace".parse()?);

    match std::env::var("RUST_LOG") {
        Ok(log) => {
            env_filter = env_filter.add_directive(log.parse()?);
        }
        Err(_) => {
            env_filter = env_filter.add_directive(Level::INFO.into());
        }
    }

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    Ok(())
}

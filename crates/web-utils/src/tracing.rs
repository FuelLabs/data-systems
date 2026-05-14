use tracing::Level;
use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_tracing() -> anyhow::Result<()> {
    let env_filter = if std::env::var_os("RUST_LOG").is_some() {
        EnvFilter::try_from_default_env()?
    } else {
        EnvFilter::new(Level::INFO.to_string())
    }
    .add_directive("fuel_web_utils::server=trace".parse()?);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    Ok(())
}

use clap::Parser;
use fuel_streams::FuelNetwork;

#[derive(Clone, Parser)]
pub struct Cli {
    /// Fuel Network to connect to.
    #[arg(
        long,
        value_name = "NETWORK",
        env = "NETWORK",
        default_value = "Local",
        value_parser = clap::value_parser!(FuelNetwork)
    )]
    pub network: FuelNetwork,
    /// Api Key for the websocket server.
    #[arg(
        long,
        value_name = "API_KEY",
        env = "API_KEY",
        default_value = "",
        help = "Api Key for the ws server."
    )]
    pub api_key: String,
    /// Maximum subscriptions for load testing
    #[arg(
        long,
        value_name = "MAXS",
        env = "MAX_SUBS",
        default_value = "10",
        help = "Maximum subscriptions for load testing."
    )]
    pub max_subscriptions: u16,
    /// Maximum step size for load testing
    #[arg(
        long,
        value_name = "SSIZE",
        env = "STEP_SIZE",
        default_value = "1",
        help = "Maximum step size for load testing."
    )]
    pub step_size: u16,
}

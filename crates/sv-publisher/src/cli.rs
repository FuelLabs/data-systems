//! This binary subscribes to events emitted from a Fuel client or node
//! to publish streams that can consumed via the `fuel-streams` SDK.

use clap::Parser;

/// CLI structure for parsing command-line arguments.
///
/// - `network`: The fuel network we want to connect to.
/// - `fuel_core_config`: Configuration for the Fuel Core service, parsed using a flattened command.
#[derive(Clone, Parser)]
pub struct Cli {
    /// API port number
    #[arg(
        long,
        value_name = "PORT",
        env = "PORT",
        default_value = "9003",
        help = "Port number for the API server"
    )]
    pub port: u16,
    /// Flattened command structure for Fuel Core configuration.
    #[command(flatten)]
    pub fuel_core_config: fuel_core_bin::cli::run::Command,
    /// Fuel Network to connect to.
    #[arg(
        long,
        value_name = "NATS_URL",
        env = "NATS_URL",
        default_value = "localhost:4222",
        help = "NATS URL to connect to."
    )]
    pub nats_url: String,
    /// Use metrics
    #[arg(
        long,
        env = "USE_METRICS",
        default_value = "false",
        help = "Enable metrics"
    )]
    pub use_metrics: bool,
}

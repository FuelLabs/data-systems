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
        value_name = "TELEMETRY_PORT",
        env = "TELEMETRY_PORT",
        default_value = "8080",
        help = "Port number for the API server"
    )]
    pub telemetry_port: u16,
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
    /// Database URL to connect to.
    #[arg(
        long,
        value_name = "DATABASE_URL",
        env = "DATABASE_URL",
        default_value = "postgresql://root@localhost:26257/defaultdb?sslmode=disable",
        help = "Database URL to connect to."
    )]
    pub db_url: String,
    /// Database URL to connect to.
    #[arg(
        long,
        value_name = "DATABASE_URL_READ",
        env = "DATABASE_URL_READ",
        help = "ReadOnly Database URL to connect to, if not provided use the same as DATABASE_URL."
    )]
    pub db_url_read: Option<String>,
    /// Start from block height
    #[arg(
        long,
        value_name = "FROM_BLOCK",
        default_value = "0",
        help = "Start from block height"
    )]
    pub from_block: u64,
    /// Use metrics
    #[arg(
        long,
        env = "USE_METRICS",
        default_value = "false",
        help = "Enable metrics"
    )]
    pub use_metrics: bool,
    /// Historical gap processing interval in seconds
    #[arg(
        long,
        value_name = "HISTORY_INTERVAL",
        env = "HISTORY_INTERVAL",
        default_value = "0",
        help = "Interval in seconds for processing historical gaps (default: 0 - disabled)."
    )]
    pub history_interval: u64,
}

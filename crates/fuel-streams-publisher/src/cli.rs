//! This binary subscribes to events emitted from a Fuel client or node
//! to publish streams that can consumed via the `fuel-streams` SDK.

use clap::Parser;
use fuel_streams::types::FuelNetwork;

/// CLI structure for parsing command-line arguments.
///
/// - `network`: The fuel network we want to connect to.
/// - `fuel_core_config`: Configuration for the Fuel Core service, parsed using a flattened command.
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
    /// Flattened command structure for Fuel Core configuration.
    #[command(flatten)]
    pub fuel_core_config: fuel_core_bin::cli::run::Command,
    /// Http server address
    #[arg(
        long,
        value_name = "TPORT",
        env = "TELEMETRY_PORT",
        default_value = "8080",
        help = "Port for the Actix Web server to bind telemetry to."
    )]
    pub telemetry_port: u16,
}

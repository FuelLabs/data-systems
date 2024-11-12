//! This binary subscribes to events emitted from a Fuel client or node
//! to publish streams that can consumed via the `fuel-streams` SDK.
use std::net::SocketAddr;

use clap::Parser;

/// CLI structure for parsing command-line arguments.
///
/// - `nats_url`: The URL of the NATS server to connect to.
/// - `fuel_core_config`: Configuration for the Fuel Core service, parsed using a flattened command.
#[derive(Clone, Parser)]
pub struct Cli {
    /// Nats connection url
    #[arg(
        long,
        value_name = "URL",
        env = "NATS_URL",
        default_value = "localhost:4222"
    )]
    pub nats_url: String,
    /// Flattened command structure for Fuel Core configuration.
    #[command(flatten)]
    pub fuel_core_config: fuel_core_bin::cli::run::Command,
    /// Http server address
    #[arg(
        long,
        value_name = "ADDR",
        env = "SERVER_ADDR",
        default_value = "0.0.0.0:8080",
        help = "Address for the Actix Web server to bind to."
    )]
    pub server_addr: SocketAddr,
}

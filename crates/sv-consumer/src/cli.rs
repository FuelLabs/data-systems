use clap::Parser;

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
    /// Fuel Network to connect to.
    #[arg(
        long,
        value_name = "NATS_URL",
        env = "NATS_URL",
        default_value = "localhost:4222",
        help = "NATS URL to connect to."
    )]
    pub nats_url: String,
    /// Nats publisher URL
    #[arg(
        long,
        value_name = "NATS_PUBLISHER_URL",
        env = "NATS_PUBLISHER_URL",
        default_value = "localhost:4333",
        help = "NATS Publisher URL to connect to."
    )]
    pub nats_publisher_url: String,
    /// Use metrics
    #[arg(
        long,
        env = "USE_METRICS",
        default_value = "false",
        help = "Enable metrics"
    )]
    pub use_metrics: bool,
}

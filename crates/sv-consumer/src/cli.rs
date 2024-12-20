use clap::Parser;

#[derive(Clone, Parser)]
pub struct Cli {
    /// Fuel Network to connect to.
    #[arg(
        long,
        value_name = "NATS_URL",
        env = "NATS_URL",
        default_value = "localhost:4222",
        help = "NATS URL to connect to."
    )]
    pub nats_url: String,
    #[arg(
        long,
        value_name = "NATS_PUBLISHER_URL",
        env = "NATS_PUBLISHER_URL",
        default_value = "localhost:4333",
        help = "NATS Publisher URL to connect to."
    )]
    pub nats_publisher_url: String,
}

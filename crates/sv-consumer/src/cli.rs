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
        short,
        value_name = "SERVICE_NAME",
        env = "SERVICE_NAME",
        default_value = "sv-emitter"
    )]
    pub service_name: String,
}

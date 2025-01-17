use clap::Parser;

#[derive(Clone, Parser)]
pub struct Cli {
    /// API port number
    #[arg(
        long,
        value_name = "PORT",
        env = "PORT",
        default_value = "8080",
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
    /// Database URL to connect to.
    #[arg(
        long,
        value_name = "DATABASE_URL",
        env = "DATABASE_URL",
        default_value = "postgresql://root@localhost:26257/defaultdb?sslmode=disable",
        help = "Database URL to connect to."
    )]
    pub db_url: String,
    /// Use metrics
    #[arg(
        long,
        env = "USE_METRICS",
        default_value = "false",
        help = "Enable metrics"
    )]
    pub use_metrics: bool,
}

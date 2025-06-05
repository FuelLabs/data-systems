use clap::Parser;

/// CLI structure for parsing command-line arguments.
#[derive(Clone, Parser)]
pub struct Cli {
    /// API port number
    #[arg(
        long,
        value_name = "PORT",
        env = "PORT",
        default_value = "9004",
        help = "Port number for the API server"
    )]
    pub port: u16,

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
        help = "ReadOnly Database URL to connect to, if not provided use the same as DATABASE_URL"
    )]
    pub db_url_read: Option<String>,

    /// Use metrics
    #[arg(
        long,
        env = "USE_METRICS",
        default_value = "false",
        help = "Enable metrics"
    )]
    pub use_metrics: bool,
}

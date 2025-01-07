use clap::Parser;

/// CLI structure for parsing command-line arguments.
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

    /// NATS URL
    #[arg(
        long,
        value_name = "NATS_URL",
        env = "NATS_URL",
        default_value = "nats://localhost:4222",
        help = "NATS URL"
    )]
    pub nats_url: String,

    /// JWT secret
    #[arg(
        long,
        value_name = "JWT_AUTH_SECRET",
        env = "JWT_AUTH_SECRET",
        default_value = "secret",
        help = "Secret key for JWT authentication"
    )]
    pub jwt_secret: String,

    /// Use metrics
    #[arg(
        long,
        env = "USE_METRICS",
        default_value = "false",
        help = "Enable metrics"
    )]
    pub use_metrics: bool,

    /// Database URL to connect to.
    #[arg(
        long,
        value_name = "DATABASE_URL",
        env = "DATABASE_URL",
        default_value = "postgresql://root@localhost:26257/defaultdb?sslmode=disable",
        help = "Database URL to connect to."
    )]
    pub db_url: String,
}

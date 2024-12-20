use clap::Parser;

/// CLI structure for parsing command-line arguments.
#[derive(Clone, Parser)]
pub struct Cli {
    /// API port number
    #[arg(
        long,
        value_name = "PORT",
        env = "API_PORT",
        default_value = "9003",
        help = "Port number for the API server"
    )]
    pub api_port: u16,

    /// NATS URL
    #[arg(
        long,
        value_name = "NATS_URL",
        env = "NATS_URL",
        default_value = "nats://localhost:4222",
        help = "NATS URL"
    )]
    pub nats_url: String,

    /// Enable S3
    #[arg(
        long,
        value_name = "AWS_S3_ENABLED",
        env = "AWS_S3_ENABLED",
        default_value = "true",
        help = "Enable S3 integration"
    )]
    pub s3_enabled: bool,

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
}

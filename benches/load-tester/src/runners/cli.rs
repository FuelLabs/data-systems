use clap::Parser;

#[derive(Clone, Parser)]
pub struct Cli {
    /// NATS URL
    #[arg(
        long,
        value_name = "NATS_URL",
        env = "NATS_URL",
        default_value = "nats://localhost:4222",
        help = "NATS URL"
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
    /// Maximum subscriptions for load testing
    #[arg(
        long,
        value_name = "MAXS",
        env = "MAX_SUBS",
        default_value = "10",
        help = "Maximum subscriptions for load testing."
    )]
    pub max_subscriptions: u16,
    /// Maximum step size for load testing
    #[arg(
        long,
        value_name = "SSIZE",
        env = "STEP_SIZE",
        default_value = "1",
        help = "Maximum step size for load testing."
    )]
    pub step_size: u16,
}

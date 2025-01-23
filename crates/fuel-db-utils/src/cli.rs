use clap::Parser;

/// CLI structure for parsing command-line arguments.
#[derive(Clone, Parser)]
pub struct Cli {
    /// API keys number
    #[arg(
        long,
        value_name = "NKEYS",
        env = "NKEYS",
        default_value = "10",
        help = "Number of api keys to generate"
    )]
    pub nkeys: i32,

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

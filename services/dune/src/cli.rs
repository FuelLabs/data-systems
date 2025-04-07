use clap::Parser;

#[derive(Clone, Parser)]
pub struct Cli {
    #[arg(
        long,
        value_name = "NETWORK",
        env = "NETWORK",
        default_value = "local",
        help = "Network to connect to. Options are 'local', 'testnet', 'mainnet', or 'staging'."
    )]
    pub network: String,

    #[arg(
        long,
        value_name = "DATABASE_URL",
        env = "DATABASE_URL",
        default_value = "postgresql://root@localhost:26257/defaultdb?sslmode=disable",
        help = "Database URL to connect to."
    )]
    pub db_url: String,

    #[arg(long, value_name = "STORAGE_FILE_DIR", env = "STORAGE_FILE_DIR")]
    pub storage_file_dir: Option<String>,

    #[arg(
        long,
        value_name = "STORAGE_TYPE",
        env = "STORAGE_TYPE",
        default_value = "File",
        help = "Type of storage to use. Options are 'S3' or 'File'."
    )]
    pub storage_type: String,

    #[arg(
        long,
        value_name = "MAX_BLOCKS_TO_STORE",
        env = "MAX_BLOCKS_TO_STORE"
    )]
    pub max_blocks_to_store: Option<u16>,
}

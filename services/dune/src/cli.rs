use clap::Parser;
use fuel_streams_types::BlockHeight;

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
    pub max_blocks_to_store: Option<usize>,

    #[arg(long, value_name = "FROM_BLOCK", env = "FROM_BLOCK")]
    pub from_block: Option<BlockHeight>,

    #[arg(
        long,
        value_name = "BATCH_SIZE",
        env = "BATCH_SIZE",
        default_value = "3600"
    )]
    pub batch_size: usize,
}

use crate::{block_buffer::BufferType, processor::StorageTypeConfig};
use clap::Parser;
use url::Url;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[arg(long, env)]
    pub url: Url,

    #[arg(long, env)]
    pub starting_block: u32,

    #[arg(
        long,
        value_name = "STORAGE_TYPE",
        env = "STORAGE_TYPE",
        default_value = "StorageTypeConfig::File",
        help = "Type of storage to use. Options are 'S3' or 'File'."
    )]
    pub storage_type: StorageTypeConfig,

    #[arg(
        long,
        value_name = "BUFFER_TYPE",
        env = "BUFFER_TYPE",
        default_value = "disk",
        help = "Type of buffer to use for accumulating blocks. Options are 'memory' or 'disk'. \
                Memory is faster but uses more RAM. Disk uses temporary files to reduce memory usage."
    )]
    pub buffer_type: BufferType,

    #[arg(long, env, default_value = "3600")]
    pub batch_size: usize,

    /// The number of blocks to fetch in each request to the node.
    #[arg(long, env, default_value = "10")]
    pub blocks_request_batch_size: usize,

    /// The number of concurrent requests for blocks.
    #[arg(long, env, default_value = "100")]
    pub blocks_request_concurrency: usize,

    /// The number of unordered pending blocks to buffer.
    #[arg(long, env, default_value = "10000")]
    pub pending_blocks: usize,
}

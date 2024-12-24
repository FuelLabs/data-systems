// TODO: Introduce Adapters for Transient and FileStorage (NATS and S3 clients would implement those)
pub mod s3;
pub mod storage;
pub mod storage_config;

pub use s3::*;
pub use storage::*;
pub use storage_config::*;

// TODO: Introduce Adapters for Transient and FileStorage (NATS and S3 clients would implement those)

pub mod nats;
pub mod s3;

pub use nats::*;
pub use s3::*;

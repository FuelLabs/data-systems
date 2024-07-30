use serde::Deserialize;
use strum::{AsRefStr, EnumIter, EnumString};

/// Compression types
#[derive(
    Debug,
    Copy,
    Clone,
    Deserialize,
    EnumString,
    AsRefStr,
    EnumIter,
    strum_macros::Display,
)]
pub enum CompressionType {
    /// No compression to be applied
    #[strum(serialize = "none")]
    None,
    /// Zlib compression
    #[strum(serialize = "zlib")]
    Zlib,
    /// Gzip compression
    #[strum(serialize = "gzip")]
    Gzip,
    /// Brotli compression
    #[strum(serialize = "brotli")]
    Brotli,
    /// Bz compression
    #[strum(serialize = "bz")]
    Bz,
    #[strum(serialize = "lzma")]
    /// Lzma compression
    Lzma,
    #[strum(serialize = "deflate")]
    /// Deflate compression
    Deflate,
    #[strum(serialize = "zstd")]
    /// Zstd compression
    Zstd,
}

/// Serialization types
#[derive(
    Debug,
    Copy,
    Clone,
    Deserialize,
    EnumString,
    AsRefStr,
    EnumIter,
    strum_macros::Display,
)]
pub enum SerializationType {
    /// Bincode serialization
    #[strum(serialize = "bincode")]
    Bincode,
    /// Postcard serialization
    #[strum(serialize = "postcard")]
    Postcard,
}

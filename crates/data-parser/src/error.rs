#![allow(dead_code)]

use displaydoc::Display as DisplayDoc;
use thiserror::Error;

/// Password hashing error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum CompressionError {
    /// Compression/Decompression zlib error
    Zlib(std::io::Error),
    /// Compression/Decompression gzip error
    Gzip(std::io::Error),
    /// Compression/Decompression brotli error
    Brotli(std::io::Error),
    /// Compression/Decompression bz error
    Bz(std::io::Error),
    /// Compression/Decompression lzma error
    Lzma(std::io::Error),
    /// Compression/Decompression deflate error
    Deflate(std::io::Error),
    /// Compression/Decompression zstd error
    Zstd(std::io::Error),
}

/// Password hashing error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum SerdeError {
    /// serde bincode error
    Bincode(#[from] bincode::ErrorKind),
    /// serde postcard error
    Postcard(#[from] postcard::Error),
    /// serde prost encode error
    ProstEncode(#[from] prost::EncodeError),
    /// serde prost decode error
    ProstDecode(#[from] prost::DecodeError),
    /// serde json error
    Json(#[from] serde_json::Error),
}

/// Password hashing error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// compression error: {0}
    Compression(#[from] CompressionError),
    /// serde error: {0}
    Serde(#[from] SerdeError),
}

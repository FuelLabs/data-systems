#![allow(dead_code)]

use displaydoc::Display as DisplayDoc;
use thiserror::Error;

/// Compression error types
#[derive(Debug, DisplayDoc, Error)]
pub enum CompressionError {
    #[cfg(feature = "zlib")]
    /// Failed to compress or decompress data using zlib: {0}
    Zlib(std::io::Error),
    #[cfg(feature = "gzip")]
    /// Failed to compress or decompress data using gzip: {0}
    Gzip(std::io::Error),
    #[cfg(feature = "brotli")]
    /// Failed to compress or decompress data using brotli: {0}
    Brotli(std::io::Error),
    #[cfg(feature = "bzip2")]
    /// Failed to compress or decompress data using bzip2: {0}
    Bz(std::io::Error),
    #[cfg(feature = "lzma")]
    /// Failed to compress or decompress data using lzma: {0}
    Lzma(std::io::Error),
    #[cfg(feature = "deflate")]
    /// Failed to compress or decompress data using deflate: {0}
    Deflate(std::io::Error),
    #[cfg(feature = "zstd")]
    /// Failed to compress or decompress data using zstd: {0}
    Zstd(std::io::Error),
}

/// Serialization/Deserialization error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum SerdeError {
    #[cfg(feature = "bincode")]
    /// Failed to serialize or deserialize data using bincode: {0}
    Bincode(#[from] bincode::ErrorKind),
    #[cfg(feature = "postcard")]
    /// Failed to serialize or deserialize data using postcard: {0}
    Postcard(#[from] postcard::Error),
    #[cfg(feature = "json")]
    /// Failed to serialize or deserialize data using JSON: {0}
    Json(#[from] serde_json::Error),
}

/// Data parser error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum DataParserError {
    /// An error occurred during data compression or decompression: {0}
    Compression(#[from] CompressionError),
    /// An error occurred during data serialization or deserialization: {0}
    Serde(#[from] SerdeError),
    /// An error occurred during data encoding: {0}
    Encode(#[source] SerdeError),
    /// An error occurred during data decoding: {0}
    Decode(#[source] SerdeError),
    #[cfg(feature = "json")]
    /// An error occurred during data encoding to JSON: {0}
    EncodeJson(#[source] SerdeError),
    #[cfg(feature = "json")]
    /// An error occurred during data decoding from JSON: {0}
    DecodeJson(#[source] SerdeError),
}

#![allow(dead_code)]

use displaydoc::Display as DisplayDoc;
use thiserror::Error;

/// Compression error types
#[derive(Debug, DisplayDoc, Error)]
pub enum CompressionError {
    /// Failed to compress or decompress data using zlib: {0}
    Zlib(std::io::Error),
    /// Failed to compress or decompress data using gzip: {0}
    Gzip(std::io::Error),
    /// Failed to compress or decompress data using brotli: {0}
    Brotli(std::io::Error),
    /// Failed to compress or decompress data using bzip2: {0}
    Bz(std::io::Error),
    /// Failed to compress or decompress data using lzma: {0}
    Lzma(std::io::Error),
    /// Failed to compress or decompress data using deflate: {0}
    Deflate(std::io::Error),
    /// Failed to compress or decompress data using zstd: {0}
    Zstd(std::io::Error),
}

/// Serialization/Deserialization error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum SerdeError {
    /// Failed to serialize or deserialize data using bincode: {0}
    Bincode(#[from] bincode::ErrorKind),
    /// Failed to serialize or deserialize data using postcard: {0}
    Postcard(#[from] postcard::Error),
    /// Failed to serialize or deserialize data using JSON: {0}
    Json(#[from] serde_json::Error),
    /// Failed to serialize data using protobuf: {0}
    ProstEncode(#[from] prost::EncodeError),
    /// Failed to deserialize data using protobuf: {0}
    ProstDecode(#[from] prost::DecodeError),
}

/// Data parser error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// An error occurred during data compression or decompression: {0}
    Compression(#[from] CompressionError),
    /// An error occurred during data serialization or deserialization: {0}
    Serde(#[from] SerdeError),
}

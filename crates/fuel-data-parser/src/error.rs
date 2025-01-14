#![allow(dead_code)]

/// Compression error types
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[cfg(feature = "zlib")]
    #[error("Failed to compress or decompress data using zlib: {0}")]
    Zlib(std::io::Error),
    #[cfg(feature = "gzip")]
    #[error("Failed to compress or decompress data using gzip: {0}")]
    Gzip(std::io::Error),
    #[cfg(feature = "brotli")]
    #[error("Failed to compress or decompress data using brotli: {0}")]
    Brotli(std::io::Error),
    #[cfg(feature = "bzip2")]
    #[error("Failed to compress or decompress data using bzip2: {0}")]
    Bz(std::io::Error),
    #[cfg(feature = "lzma")]
    #[error("Failed to compress or decompress data using lzma: {0}")]
    Lzma(std::io::Error),
    #[cfg(feature = "deflate")]
    #[error("Failed to compress or decompress data using deflate: {0}")]
    Deflate(std::io::Error),
    #[cfg(feature = "zstd")]
    #[error("Failed to compress or decompress data using zstd: {0}")]
    Zstd(std::io::Error),
}

/// Serialization/Deserialization error types.
#[derive(Debug, thiserror::Error)]
pub enum SerdeError {
    #[cfg(feature = "bincode")]
    #[error(transparent)]
    Bincode(#[from] bincode::ErrorKind),
    #[cfg(feature = "postcard")]
    #[error(transparent)]
    Postcard(#[from] postcard::Error),
    #[cfg(feature = "json")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Data parser error types.
#[derive(Debug, thiserror::Error)]
pub enum DataParserError {
    #[error(transparent)]
    Compression(#[from] CompressionError),
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error("An error occurred during data encoding: {0}")]
    Encode(#[source] SerdeError),
    #[error("An error occurred during data decoding: {0}")]
    Decode(#[source] SerdeError),
    #[cfg(feature = "json")]
    #[error("An error occurred during data encoding to JSON: {0}")]
    EncodeJson(#[source] SerdeError),
    #[cfg(feature = "json")]
    #[error("An error occurred during data decoding from JSON: {0}")]
    DecodeJson(#[source] SerdeError),
}

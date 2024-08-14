use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};

/// Compression types
#[derive(
    Debug,
    Copy,
    Clone,
    EnumString,
    AsRefStr,
    EnumIter,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
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
    EnumString,
    AsRefStr,
    EnumIter,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum_macros::Display,
)]
pub enum SerializationType {
    /// Bincode serialization
    #[strum(serialize = "bincode")]
    Bincode,
    /// Postcard serialization
    #[strum(serialize = "postcard")]
    Postcard,
    /// json serialization
    #[strum(serialize = "json")]
    Json,
}

/// nats formatted internal message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NatsInternalMessage {
    pub subject: String,
    pub timestamp: String,
    pub data: Vec<u8>,
}

impl NatsInternalMessage {
    pub fn new(subject: &str, data: Vec<u8>) -> Self {
        let now: DateTime<Utc> = Utc::now();
        // Formatting the datetime as an ISO 8601 string
        let timestamp = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        Self {
            subject: subject.to_string(),
            timestamp,
            data,
        }
    }

    pub fn json_serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn deserialize_from_json(
        serialized: &[u8],
    ) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(serialized)
    }
}

/// nats formatted user message
#[derive(Debug, Clone, Serialize)]
pub struct NatsFormattedMessage<T: serde::de::DeserializeOwned + Clone> {
    pub subject: String,
    pub timestamp: String,
    pub data: T,
}

impl<T> NatsFormattedMessage<T>
where
    T: serde::de::DeserializeOwned + Clone,
{
    pub fn ts_as_millis(&self) -> u128 {
        DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()
            .map(|ts| ts.timestamp_millis() as u128)
            .unwrap_or_else(|| Utc::now().timestamp_millis() as u128)
    }
}

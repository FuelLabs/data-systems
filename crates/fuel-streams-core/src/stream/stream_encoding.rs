use std::fmt::Debug;

use async_trait::async_trait;
use fuel_data_parser::{DataParseable, DataParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamData<T> {
    pub subject: String,
    pub timestamp: String,
    /// The payload published for the subject
    pub payload: T,
}

impl<T> StreamData<T>
where
    T: serde::de::DeserializeOwned + Clone,
{
    pub fn new(subject: &str, payload: T) -> Self {
        let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        // Formatting the datetime as an ISO 8601 string
        let timestamp = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        Self {
            subject: subject.to_string(),
            timestamp,
            payload,
        }
    }

    #[cfg(feature = "bench-helpers")]
    pub fn ts_as_millis(&self) -> u128 {
        use chrono::{DateTime, Utc};

        DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()
            .map(|ts| ts.timestamp_millis() as u128)
            .unwrap_or_else(|| Utc::now().timestamp_millis() as u128)
    }
}

#[async_trait]
pub trait StreamEncoder: DataParseable {
    // TODO: Should we remove the `StreamData` type and encode/decode the raw data only
    fn encode(&self, subject: &str) -> Vec<u8> {
        let data = StreamData::new(subject, self.clone());

        Self::data_parser()
            .encode_json(&data)
            .expect("Streamable must encode correctly")
    }

    fn decode(encoded: Vec<u8>) -> Self {
        Self::decode_raw(encoded).payload
    }

    fn decode_raw(encoded: Vec<u8>) -> StreamData<Self> {
        Self::data_parser()
            .decode_json(&encoded)
            .expect("Streamable must decode correctly")
    }

    fn data_parser() -> DataParser {
        DataParser::default()
    }
}

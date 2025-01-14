#![doc = include_str!("../README.md")]

mod compression_strategies;
mod error;

use std::{fmt::Debug, sync::Arc};

pub use compression_strategies::*;
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;

pub use crate::error::{CompressionError, DataParserError, SerdeError};

/// Serialization types supported for data parsing
#[derive(Debug, Clone, strum::EnumIter, strum_macros::Display)]
pub enum SerializationType {
    /// Bincode serialization
    #[cfg(feature = "bincode")]
    #[strum(serialize = "bincode")]
    Bincode,
    /// json serialization
    #[cfg(feature = "json")]
    #[strum(serialize = "json")]
    Json,
    /// Postcard serialization
    #[cfg(feature = "postcard")]
    #[strum(serialize = "postcard")]
    Postcard,
}

#[async_trait::async_trait]
pub trait DataEncoder:
    serde::Serialize
    + serde::de::DeserializeOwned
    + Clone
    + Send
    + Sync
    + Debug
    + std::marker::Sized
{
    type Err: std::error::Error + From<DataParserError>;

    fn data_parser() -> DataParser {
        DataParser::default()
    }

    async fn encode(&self) -> Result<Vec<u8>, Self::Err> {
        Self::data_parser().encode(self).await.map_err(Into::into)
    }

    #[cfg(feature = "json")]
    fn encode_json(&self) -> Result<Vec<u8>, Self::Err> {
        Self::data_parser().encode_json(self).map_err(Into::into)
    }

    async fn decode(encoded: &[u8]) -> Result<Self, Self::Err> {
        Self::data_parser()
            .decode(encoded)
            .await
            .map_err(Into::into)
    }

    #[cfg(feature = "json")]
    fn decode_json(encoded: &[u8]) -> Result<Self, Self::Err> {
        Self::data_parser().decode_json(encoded).map_err(Into::into)
    }

    #[cfg(feature = "json")]
    fn to_json_value(&self) -> Result<serde_json::Value, Self::Err> {
        Self::data_parser().to_json_value(self).map_err(Into::into)
    }
}

/// `DataParser` is a utility struct for encoding (serializing and optionally compressing)
/// and decoding (deserializing and optionally decompressing) data. It is useful for
/// optimizing memory usage and I/O bandwidth by applying different
/// serialization formats and optional compression strategies.
///
/// # Fields
///
/// * `compression_strategy` - An `Option<Arc<dyn CompressionStrategy>>` that defines
///   the method of data compression. If `None`, no compression is applied.
/// * `serialization_type` - An enum that specifies the serialization format
///   (e.g., Bincode, Postcard, JSON).
///
/// # Examples
///
/// ```
/// use fuel_data_parser::*;
/// use std::sync::Arc;
///
/// #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// struct TestData {
///     field: String,
/// }
///
/// impl DataEncoder for TestData {
///     type Err = DataParserError;
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let parser = DataParser::default();
///
///     let original_data = TestData { field: "test".to_string() };
///     let encoded = parser.encode(&original_data).await?;
///     let decoded: TestData = parser.decode(&encoded).await?;
///
///     assert_eq!(original_data, decoded);
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct DataParser {
    compression_strategy: Option<Arc<dyn CompressionStrategy>>,
    pub serialization_type: SerializationType,
}

impl Default for DataParser {
    /// Provides a default instance of `DataParser` with no compression strategy
    /// and `SerializationType::Json`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::{DataParser, SerializationType};
    ///
    /// let parser = DataParser::default();
    /// assert!(matches!(parser.serialization_type, SerializationType::Json));
    /// ```
    fn default() -> Self {
        Self {
            compression_strategy: None,
            serialization_type: SerializationType::Json,
        }
    }
}

impl DataParser {
    /// Sets the compression strategy for the `DataParser`.
    ///
    /// # Arguments
    ///
    /// * `compression_strategy` - A reference to an `Arc` of a `CompressionStrategy` trait object.
    ///
    /// # Returns
    ///
    /// A new instance of `DataParser` with the updated compression strategy.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::*;
    /// use std::sync::Arc;
    ///
    /// let parser = DataParser::default()
    ///     .with_compression_strategy(&DEFAULT_COMPRESSION_STRATEGY);
    /// ```
    pub fn with_compression_strategy(
        mut self,
        compression_strategy: &Arc<dyn CompressionStrategy>,
    ) -> Self {
        self.compression_strategy = Some(compression_strategy.clone());
        self
    }

    /// Sets the serialization type for the `DataParser`.
    ///
    /// # Arguments
    ///
    /// * `serialization_type` - A `SerializationType` enum specifying the desired serialization format.
    ///
    /// # Returns
    ///
    /// A new instance of `DataParser` with the updated serialization type.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::*;
    ///
    /// let parser = DataParser::default()
    ///     .with_serialization_type(SerializationType::Postcard);
    /// ```
    pub fn with_serialization_type(
        mut self,
        serialization_type: SerializationType,
    ) -> Self {
        self.serialization_type = serialization_type;
        self
    }

    /// Encodes the provided data by serializing and optionally compressing it.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to a data structure implementing the `DataParseable` trait.
    ///
    /// # Returns
    ///
    /// A `Result` containing either a `Vec<u8>` of the serialized (and optionally compressed) data,
    /// or an `Error` if encoding fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::*;
    ///
    /// #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    /// struct TestData {
    ///     field: String,
    /// }
    ///
    /// impl DataEncoder for TestData {
    ///     type Err = DataParserError;
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let parser = DataParser::default();
    ///     let data = TestData { field: "test".to_string() };
    ///     let encoded = parser.encode(&data).await?;
    ///     assert!(!encoded.is_empty());
    ///     Ok(())
    /// }
    /// ```
    pub async fn encode<T: DataEncoder>(
        &self,
        data: &T,
    ) -> Result<Vec<u8>, DataParserError> {
        let serialized_data = self.serialize(data).await?;
        Ok(match &self.compression_strategy {
            Some(strategy) => strategy.compress(&serialized_data[..]).await?,
            None => serialized_data,
        })
    }

    #[cfg(feature = "json")]
    pub fn encode_json<T: DataEncoder>(
        &self,
        data: &T,
    ) -> Result<Vec<u8>, DataParserError> {
        self.serialize_json(data)
    }

    #[cfg(feature = "json")]
    pub fn to_json_value<T: serde::Serialize>(
        &self,
        data: &T,
    ) -> Result<serde_json::Value, DataParserError> {
        self.serialize_json_value(data)
    }

    /// Serializes the provided data according to the selected `SerializationType`.
    ///
    /// # Arguments
    ///
    /// * `raw_data` - A reference to a data structure implementing the `DataParseable` trait.
    ///
    /// # Returns
    ///
    /// A `Result` containing either a `Vec<u8>` of the serialized data,
    /// or an `Error` if serialization fails.
    pub async fn serialize<T: DataEncoder>(
        &self,
        raw_data: &T,
    ) -> Result<Vec<u8>, DataParserError> {
        match self.serialization_type {
            #[cfg(feature = "bincode")]
            SerializationType::Bincode => bincode::serialize(&raw_data)
                .map_err(|e| DataParserError::Encode(SerdeError::Bincode(*e))),
            #[cfg(feature = "json")]
            SerializationType::Json => serde_json::to_vec(&raw_data)
                .map_err(|e| DataParserError::EncodeJson(SerdeError::Json(e))),
            #[cfg(feature = "postcard")]
            SerializationType::Postcard => postcard::to_allocvec(&raw_data)
                .map_err(|e| DataParserError::Encode(SerdeError::Postcard(e))),
        }
    }

    #[cfg(feature = "json")]
    fn serialize_json<T: DataEncoder>(
        &self,
        raw_data: &T,
    ) -> Result<Vec<u8>, DataParserError> {
        serde_json::to_vec(&raw_data)
            .map_err(|e| DataParserError::EncodeJson(SerdeError::Json(e)))
    }

    #[cfg(feature = "json")]
    fn serialize_json_value<T: serde::Serialize>(
        &self,
        raw_data: &T,
    ) -> Result<serde_json::Value, DataParserError> {
        serde_json::to_value(raw_data)
            .map_err(|e| DataParserError::EncodeJson(SerdeError::Json(e)))
    }

    /// Decodes the provided data by deserializing and optionally decompressing it.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice (`&[u8]`) representing the serialized (and optionally compressed) data.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the deserialized data structure,
    /// or an `Error` if decoding fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::*;
    ///
    /// #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    /// struct TestData {
    ///     field: String,
    /// }
    ///
    /// impl DataEncoder for TestData {
    ///     type Err = DataParserError;
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let parser = DataParser::default();
    ///     let original_data = TestData { field: "test".to_string() };
    ///     let encoded = parser.encode(&original_data).await?;
    ///     let decoded: TestData = parser.decode(&encoded).await?;
    ///     assert_eq!(original_data, decoded);
    ///     Ok(())
    /// }
    /// ```
    pub async fn decode<T: DataEncoder>(
        &self,
        data: &[u8],
    ) -> Result<T, DataParserError> {
        let data = match &self.compression_strategy {
            Some(strategy) => strategy.decompress(data).await?,
            None => data.to_vec(),
        };
        let decoded_data = self.deserialize(&data[..])?;
        Ok(decoded_data)
    }

    #[cfg(feature = "json")]
    pub fn decode_json<T: DataEncoder>(
        &self,
        data: &[u8],
    ) -> Result<T, DataParserError> {
        self.deserialize_json(data)
    }

    /// Deserializes the provided data according to the selected `SerializationType`.
    ///
    /// # Arguments
    ///
    /// * `raw_data` - A byte slice (`&[u8]`) representing the serialized data.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the deserialized data structure,
    /// or an `Error` if deserialization fails.
    pub fn deserialize<T: DeserializeOwned>(
        &self,
        raw_data: &[u8],
    ) -> Result<T, DataParserError> {
        match self.serialization_type {
            #[cfg(feature = "bincode")]
            SerializationType::Bincode => bincode::deserialize(raw_data)
                .map_err(|e| DataParserError::Decode(SerdeError::Bincode(*e))),
            #[cfg(feature = "json")]
            SerializationType::Json => self.deserialize_json(raw_data),
            #[cfg(feature = "postcard")]
            SerializationType::Postcard => postcard::from_bytes(raw_data)
                .map_err(|e| DataParserError::Decode(SerdeError::Postcard(e))),
        }
    }

    #[cfg(feature = "json")]
    fn deserialize_json<T: DeserializeOwned>(
        &self,
        raw_data: &[u8],
    ) -> Result<T, DataParserError> {
        serde_json::from_slice(raw_data)
            .map_err(|e| DataParserError::DecodeJson(SerdeError::Json(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    struct TestData {
        field: String,
    }

    impl DataEncoder for TestData {
        type Err = DataParserError;
    }

    #[tokio::test]
    async fn test_encode_decode() {
        let parser = DataParser::default();
        let original_data = TestData {
            field: "test".to_string(),
        };
        let encoded = parser.encode(&original_data).await.unwrap();
        let decoded: TestData = parser.decode(&encoded).await.unwrap();
        assert_eq!(original_data, decoded);
    }

    #[tokio::test]
    async fn test_serialization_types() {
        let data = TestData {
            field: "test".to_string(),
        };

        for serialization_type in [
            #[cfg(feature = "bincode")]
            SerializationType::Bincode,
            #[cfg(feature = "postcard")]
            SerializationType::Postcard,
            #[cfg(feature = "json")]
            SerializationType::Json,
        ] {
            let parser = DataParser::default()
                .with_serialization_type(serialization_type);
            let encoded = parser.encode(&data).await.unwrap();
            let decoded: TestData = parser.decode(&encoded).await.unwrap();
            assert_eq!(data, decoded);
        }
    }

    #[tokio::test]
    async fn test_compression_strategies() {
        let data = TestData {
            field: "test".to_string(),
        };

        for strategy in ALL_COMPRESSION_STRATEGIES.iter() {
            let parser =
                DataParser::default().with_compression_strategy(strategy);
            let encoded = parser.encode(&data).await.unwrap();
            let decoded: TestData = parser.decode(&encoded).await.unwrap();
            assert_eq!(data, decoded);
        }
    }
}

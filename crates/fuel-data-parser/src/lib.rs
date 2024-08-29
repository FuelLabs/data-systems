#![doc = include_str!("../README.md")]

mod compression_strategies;
mod error;

use std::{fmt::Debug, sync::Arc};

pub use compression_strategies::*;

pub use crate::error::{CompressionError, Error, SerdeError};

/// Serialization types supported for data parsing
#[derive(Debug, Clone, strum::EnumIter, strum_macros::Display)]
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

/// Traits required for a data type to be parseable
pub trait DataParseable:
    serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + Debug
{
}

impl<
        T: serde::Serialize
            + serde::de::DeserializeOwned
            + Clone
            + Send
            + Sync
            + Debug,
    > DataParseable for T
{
}

/// `DataParser` is a utility struct for encoding (compressing & serializing)
/// and decoding (decompressing & deserializing) data. It is useful for
/// optimizing memory usage and I/O bandwidth by applying different
/// compression strategies and serialization formats.
///
/// # Fields
///
/// * `compression_strategy` - An `Arc` to a `CompressionStrategy` trait object
///   that defines the method of data compression.
/// * `serialization_type` - An enum that specifies the serialization format
///   (e.g., Bincode, Postcard, JSON).
///
/// # Examples
///
/// ```
/// use fuel_data_parser::{DataParser, SerializationType};
/// use std::sync::Arc;
///
/// #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// struct TestData {
///     field: String,
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
    compression_strategy: Arc<dyn CompressionStrategy>,
    pub serialization_type: SerializationType,
}

impl Default for DataParser {
    /// Provides a default instance of `DataParser` with default compression strategy
    /// and `SerializationType::Postcard`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::{DataParser, SerializationType};
    ///
    /// let parser = DataParser::default();
    /// assert!(matches!(parser.serialization_type, SerializationType::Postcard));
    /// ```
    fn default() -> Self {
        Self {
            compression_strategy: DEFAULT_COMPRESSION_STRATEGY.clone(),
            serialization_type: SerializationType::Postcard,
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
    /// use fuel_data_parser::{DataParser, DEFAULT_COMPRESSION_STRATEGY};
    /// use std::sync::Arc;
    ///
    /// let parser = DataParser::default()
    ///     .with_compression_strategy(&DEFAULT_COMPRESSION_STRATEGY);
    /// ```
    pub fn with_compression_strategy(
        mut self,
        compression_strategy: &Arc<dyn CompressionStrategy>,
    ) -> Self {
        self.compression_strategy = compression_strategy.clone();
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
    /// use fuel_data_parser::{DataParser, SerializationType};
    ///
    /// let parser = DataParser::default()
    ///     .with_serialization_type(SerializationType::Json);
    /// ```
    pub fn with_serialization_type(
        mut self,
        serialization_type: SerializationType,
    ) -> Self {
        self.serialization_type = serialization_type;
        self
    }

    /// Encodes the provided data by serializing and then compressing it.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to a data structure implementing the `DataParseable` trait.
    ///
    /// # Returns
    ///
    /// A `Result` containing either a `Vec<u8>` of the compressed, serialized data,
    /// or an `Error` if encoding fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::DataParser;
    ///
    /// #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    /// struct TestData {
    ///     field: String,
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
    pub async fn encode<T: DataParseable>(
        &self,
        data: &T,
    ) -> Result<Vec<u8>, Error> {
        let serialized_data = self.serialize(data).await?;
        Ok(self
            .compression_strategy
            .compress(&serialized_data[..])
            .await?)
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
    pub async fn serialize<T: DataParseable>(
        &self,
        raw_data: &T,
    ) -> Result<Vec<u8>, Error> {
        match self.serialization_type {
            SerializationType::Bincode => bincode::serialize(&raw_data)
                .map_err(|e| Error::Serde(SerdeError::Bincode(*e))),
            SerializationType::Postcard => postcard::to_allocvec(&raw_data)
                .map_err(|e| Error::Serde(SerdeError::Postcard(e))),
            SerializationType::Json => serde_json::to_vec(&raw_data)
                .map_err(|e| Error::Serde(SerdeError::Json(e))),
        }
    }

    /// Decodes the provided data by decompressing and then deserializing it.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice (`&[u8]`) representing the compressed, serialized data.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the deserialized data structure,
    /// or an `Error` if decoding fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_data_parser::DataParser;
    ///
    /// #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    /// struct TestData {
    ///     field: String,
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
    pub async fn decode<T: DataParseable>(
        &self,
        data: &[u8],
    ) -> Result<T, Error> {
        let decompressed_data =
            self.compression_strategy.decompress(data).await?;
        let deserialized_data =
            self.deserialize(&decompressed_data[..]).await?;
        Ok(deserialized_data)
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
    pub async fn deserialize<'a, T: serde::Deserialize<'a>>(
        &self,
        raw_data: &'a [u8],
    ) -> Result<T, Error> {
        match self.serialization_type {
            SerializationType::Bincode => bincode::deserialize(raw_data)
                .map_err(|e| Error::Serde(SerdeError::Bincode(*e))),
            SerializationType::Postcard => postcard::from_bytes(raw_data)
                .map_err(|e| Error::Serde(SerdeError::Postcard(e))),
            SerializationType::Json => serde_json::from_slice(raw_data)
                .map_err(|e| Error::Serde(SerdeError::Json(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    struct TestData {
        field: String,
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
            SerializationType::Bincode,
            SerializationType::Postcard,
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
        let compression_strategies: Vec<Arc<dyn CompressionStrategy>> = vec![
            Arc::new(ZLibCompressionStrategy),
            #[cfg(feature = "bench-helpers")]
            Arc::new(GzipCompressionStrategy),
            #[cfg(feature = "bench-helpers")]
            Arc::new(BrotliCompressionStrategy),
        ];

        for strategy in compression_strategies {
            let parser =
                DataParser::default().with_compression_strategy(&strategy);
            let encoded = parser.encode(&data).await.unwrap();
            let decoded: TestData = parser.decode(&encoded).await.unwrap();
            assert_eq!(data, decoded);
        }
    }
}

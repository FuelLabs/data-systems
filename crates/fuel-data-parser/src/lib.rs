mod compression_strategies;
mod error;

use std::{fmt::Debug, sync::Arc};

pub use compression_strategies::*;

pub use crate::error::{CompressionError, Error, SerdeError};

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

#[derive(Clone)]
pub struct DataParser {
    compression_strategy: Arc<dyn CompressionStrategy>,
    serialization_type: SerializationType,
}

impl Default for DataParser {
    fn default() -> Self {
        Self {
            compression_strategy: DEFAULT_COMPRESSION_STRATEGY.clone(),
            serialization_type: SerializationType::Postcard,
        }
    }
}

impl DataParser {
    pub fn with_compression_strategy(
        mut self,
        compression_strategy: &Arc<dyn CompressionStrategy>,
    ) -> Self {
        self.compression_strategy = compression_strategy.clone();
        self
    }

    pub fn with_serialization_type(
        mut self,
        serialization_type: SerializationType,
    ) -> Self {
        self.serialization_type = serialization_type;
        self
    }

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

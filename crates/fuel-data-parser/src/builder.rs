#![allow(dead_code)]

use async_compression::Level;

use crate::{
    parser::DataParser,
    types::{CompressionType, SerializationType},
};

#[derive(Debug, Clone, Default)]
pub struct DataParserBuilder {
    compression_type: Option<CompressionType>,
    compression_level: Option<Level>,
    serialization_type: Option<SerializationType>,
}

impl DataParserBuilder {
    /// Creates a new builder instance with default settings
    pub fn new() -> Self {
        Self {
            compression_type: None,
            compression_level: None,
            serialization_type: None,
        }
    }

    /// Sets the compression type and returns the builder
    pub fn with_compression(
        mut self,
        compression_type: CompressionType,
    ) -> Self {
        self.compression_type = Some(compression_type);
        self
    }

    /// Sets the compression level and returns the builder
    pub fn with_compression_level(mut self, compression_level: Level) -> Self {
        self.compression_level = Some(compression_level);
        self
    }

    /// Sets the serialization type and returns the builder
    pub fn with_serialization(
        mut self,
        serialization_type: SerializationType,
    ) -> Self {
        self.serialization_type = Some(serialization_type);
        self
    }

    /// Builds the DataParser instance using the specified settings or defaults
    pub fn build(self) -> DataParser {
        DataParser {
            compression_type: self
                .compression_type
                .unwrap_or(CompressionType::Zlib),
            compression_level: self.compression_level.unwrap_or(Level::Best),
            serialization_type: self
                .serialization_type
                .unwrap_or(SerializationType::Bincode),
        }
    }
}

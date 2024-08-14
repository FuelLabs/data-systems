#![allow(dead_code)]
#![allow(unused)]

use std::fmt::Debug;

use async_compression::{
    tokio::write::{
        BrotliDecoder,
        BrotliEncoder,
        BzDecoder,
        BzEncoder,
        DeflateDecoder,
        DeflateEncoder,
        GzipDecoder,
        GzipEncoder,
        LzmaDecoder,
        LzmaEncoder,
        ZlibDecoder,
        ZlibEncoder,
        ZstdDecoder,
        ZstdEncoder,
    },
    Level,
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use tokio::io::AsyncWriteExt as _;

use crate::{
    error::{CompressionError, Error, SerdeError},
    types::{
        CompressionType,
        NatsFormattedMessage,
        NatsInternalMessage,
        SerializationType,
    },
};

pub trait DataParserSerializable:
    serde::Serialize + Clone + Send + Sync + Debug
{
}
impl<T: serde::Serialize + Clone + Send + Sync + Debug> DataParserSerializable
    for T
{
}

pub trait DataParserDeserializable:
    serde::de::DeserializeOwned + Clone + Send + Sync + Debug
{
}
impl<T: serde::de::DeserializeOwned + Clone + Send + Sync + Debug>
    DataParserDeserializable for T
{
}

/// Prost Message Wrapper allowing serialization/deserialization
pub struct ProstDataParser {
    data_parser: DataParser,
}

impl ProstDataParser {
    /// Constructor for ProstDataParser
    pub fn new(
        compression_type: CompressionType,
        compression_level: Level,
        serialization_type: SerializationType,
    ) -> Self {
        Self {
            data_parser: DataParser::new(
                compression_type,
                compression_level,
                serialization_type,
            ),
        }
    }

    /// Method to serialize
    fn serialize<T>(&self, data: T) -> Result<Vec<u8>, Error>
    where
        T: prost::Message + std::default::Default,
    {
        let mut buf = Vec::new();
        data.encode(&mut buf)
            .map_err(|e| Error::Serde(SerdeError::ProstEncode(e)))?;
        Ok(buf)
    }

    /// Method to deserialize
    fn deserialize<T>(buf: Vec<u8>) -> Result<T, Error>
    where
        T: prost::Message + std::default::Default,
    {
        T::decode(Bytes::from(buf))
            .map_err(|e| Error::Serde(SerdeError::ProstDecode(e)))
    }

    /// Compress the data
    async fn compress(&self, raw_data: &[u8]) -> Result<Vec<u8>, Error> {
        self.data_parser.compress(raw_data).await
    }

    /// Decompress the data
    async fn decompress(&self, raw_data: &[u8]) -> Result<Vec<u8>, Error> {
        self.data_parser.decompress(raw_data).await
    }
}

macro_rules! define_compression_methods {
    ($($name:ident),+) => {
        paste::item! {
            $(
                async fn [<compress_ $name:lower>](&self, in_data: &[u8]) -> Result<Vec<u8>, Error> {
                    let mut encoder = [<$name Encoder>]::with_quality(Vec::new(), self.compression_level);
                    encoder.write_all(in_data).await.map_err(|e| Error::Compression(CompressionError::[<$name>](e)))?;
                    encoder.shutdown().await.map_err(|e| Error::Compression(CompressionError::[<$name>](e)))?;
                    Ok(encoder.into_inner())
                }

                async fn [<decompress_ $name:lower>](&self, in_data: &[u8]) -> Result<Vec<u8>, Error> {
                    let mut decoder = [<$name Decoder>]::new(Vec::new());
                    decoder.write_all(in_data).await.map_err(|e| Error::Compression(CompressionError::[<$name>](e)))?;
                    decoder.shutdown().await.map_err(|e| Error::Compression(CompressionError::[<$name>](e)))?;
                    Ok(decoder.into_inner())
                }
            )*
        }
    };
}

/// DataParser implementation
#[derive(Clone, Debug)]
pub struct DataParser {
    pub compression_type: CompressionType,
    pub compression_level: Level,
    pub serialization_type: SerializationType,
}

impl Default for DataParser {
    fn default() -> Self {
        Self {
            compression_type: CompressionType::Zlib,
            compression_level: Level::Fastest,
            serialization_type: SerializationType::Postcard,
        }
    }
}

impl DataParser {
    /// Constructor for a new data parser
    fn new(
        compression_type: CompressionType,
        compression_level: Level,
        serialization_type: SerializationType,
    ) -> Self {
        Self {
            compression_type,
            compression_level,
            serialization_type,
        }
    }

    pub fn set_compression_type(&mut self, compression_type: CompressionType) {
        self.compression_type = compression_type;
    }

    pub fn set_compression_level(&mut self, compression_level: Level) {
        self.compression_level = compression_level;
    }

    pub fn set_serialization_type(
        &mut self,
        serialization_type: SerializationType,
    ) {
        self.serialization_type = serialization_type;
    }

    // Macro invocation to generate methods
    define_compression_methods!(Zlib, Gzip, Brotli, Bz, Lzma, Deflate, Zstd);

    #[cfg(feature = "test-helpers")]
    pub async fn test_serialize_and_compress<T: DataParserSerializable>(
        &self,
        raw_data: &T,
    ) -> Result<Vec<u8>, Error> {
        self.serialize_and_compress(raw_data).await
    }

    /// Serializes and compresses the data
    async fn serialize_and_compress<T: DataParserSerializable>(
        &self,
        raw_data: &T,
    ) -> Result<Vec<u8>, Error> {
        let serialized_data = self.serialize(raw_data).await?;
        let compressed_data = self.compress(&serialized_data[..]).await?;
        Ok(compressed_data)
    }

    #[cfg(feature = "test-helpers")]
    pub async fn test_decompress_and_deserialize<
        T: DataParserDeserializable,
    >(
        &self,
        raw_data: &[u8],
    ) -> Result<T, Error> {
        self.decompress_and_deserialize(raw_data).await
    }

    /// Decompresses and deserializes the data
    async fn decompress_and_deserialize<T: DataParserDeserializable>(
        &self,
        raw_data: &[u8],
    ) -> Result<T, Error> {
        let decompressed_data = self.decompress(raw_data).await?;
        let deserialized_data =
            self.deserialize(&decompressed_data[..]).await?;
        Ok(deserialized_data)
    }

    /// Deserialized and decompresses the data received from nats
    pub async fn from_nats_message<T: DataParserDeserializable>(
        &self,
        nats_data: Vec<u8>,
    ) -> Result<NatsFormattedMessage<T>, Error> {
        let nats_formatted_message =
            NatsInternalMessage::deserialize_from_json(&nats_data)
                .map_err(|e| Error::Serde(SerdeError::Json(e)))?;
        let original_message_data = self
            .decompress_and_deserialize::<T>(&nats_formatted_message.data)
            .await?;
        Ok(NatsFormattedMessage {
            subject: nats_formatted_message.subject,
            timestamp: nats_formatted_message.timestamp,
            data: original_message_data,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub async fn test_serialize<T: DataParserSerializable>(
        &self,
        raw_data: &T,
    ) -> Result<Vec<u8>, Error> {
        self.serialize(raw_data).await
    }

    /// Serializes the data
    async fn serialize<T: DataParserSerializable>(
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

    #[cfg(feature = "test-helpers")]
    pub async fn test_deserialize<'a, T: serde::Deserialize<'a>>(
        &self,
        raw_data: &'a [u8],
    ) -> Result<T, Error> {
        self.deserialize(raw_data).await
    }

    /// Deserializes the data
    async fn deserialize<'a, T: serde::Deserialize<'a>>(
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

    #[cfg(feature = "test-helpers")]
    pub async fn test_compress(
        &self,
        raw_data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.compress(raw_data).await
    }

    /// Compresses the data
    async fn compress(&self, raw_data: &[u8]) -> Result<Vec<u8>, Error> {
        match self.compression_type {
            CompressionType::None => Ok(raw_data.to_vec()),
            CompressionType::Zlib => self.compress_zlib(raw_data).await,
            CompressionType::Gzip => self.compress_gzip(raw_data).await,
            CompressionType::Brotli => self.compress_brotli(raw_data).await,
            CompressionType::Bz => self.compress_bz(raw_data).await,
            CompressionType::Lzma => self.compress_lzma(raw_data).await,
            CompressionType::Deflate => self.compress_deflate(raw_data).await,
            CompressionType::Zstd => self.compress_zstd(raw_data).await,
        }
    }

    #[cfg(feature = "test-helpers")]
    pub async fn test_decompress(
        &self,
        raw_data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.decompress(raw_data).await
    }

    /// Decompresses the data
    async fn decompress(&self, raw_data: &[u8]) -> Result<Vec<u8>, Error> {
        match self.compression_type {
            CompressionType::None => Ok(raw_data.to_vec()),
            CompressionType::Zlib => self.decompress_zlib(raw_data).await,
            CompressionType::Gzip => self.decompress_gzip(raw_data).await,
            CompressionType::Brotli => self.decompress_brotli(raw_data).await,
            CompressionType::Bz => self.decompress_bz(raw_data).await,
            CompressionType::Lzma => self.decompress_lzma(raw_data).await,
            CompressionType::Deflate => self.decompress_deflate(raw_data).await,
            CompressionType::Zstd => self.decompress_zstd(raw_data).await,
        }
    }

    /// Receives a subject and a serializable data to prepare a payload to send over nats
    pub async fn to_nats_payload(
        &self,
        subject: &str,
        raw_data: &impl DataParserSerializable,
    ) -> Result<Vec<u8>, Error> {
        let modified_raw_data = self.serialize_and_compress(raw_data).await?;
        let nats_internal_message =
            NatsInternalMessage::new(subject, modified_raw_data);
        let nats_internal_message_json_serialized = nats_internal_message
            .json_serialize()
            .map_err(|e| Error::Serde(SerdeError::Json(e)))?;
        Ok(nats_internal_message_json_serialized)
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use async_compression::Level;
    use fuel_core_types::{
        blockchain::block::Block,
        fuel_tx::Transaction,
        fuel_types::{AssetId, ChainId},
    };
    use fuel_streams_macros::subject::*;
    use serde::{Deserialize as SerdeDeserialize, Serialize};

    use crate::{
        builder::DataParserBuilder,
        types::{CompressionType, SerializationType},
    };

    // test data structure
    #[derive(
        Clone,
        Debug,
        Default,
        Subject,
        Serialize,
        SerdeDeserialize,
        Eq,
        PartialEq,
    )]
    #[subject_wildcard = "test.>"]
    #[subject_format = "test.{id}.{version}.{receipt}.{asset}.{chain_id}"]
    struct MyTestData {
        id: Option<String>,
        version: Option<u64>,
        receipt: Option<String>,
        asset: Option<AssetId>,
        chain_id: Option<ChainId>,
    }

    fn gen_test_data() -> MyTestData {
        MyTestData {
            id: Some("1".to_string()),
            version: Some(1u64),
            receipt: Some("receipt_1".to_string()),
            asset: Some(AssetId::zeroed()),
            chain_id: Some(ChainId::new(1)),
        }
    }

    #[tokio::test]
    async fn test_data_parser_builder() {
        let data_parser = DataParserBuilder::new()
            .with_compression(CompressionType::Gzip)
            .with_compression_level(Level::Fastest)
            .with_serialization(SerializationType::Bincode)
            .build();

        assert!(
            data_parser.compression_type == CompressionType::Gzip,
            "Compression type"
        );
        assert!(
            data_parser.serialization_type == SerializationType::Bincode,
            "Compression type"
        );
    }

    #[tokio::test]
    async fn test_data_serialization() {
        // construct the data parser
        let data_parser = DataParserBuilder::new()
            .with_compression(CompressionType::Gzip)
            .with_compression_level(Level::Fastest)
            .with_serialization(SerializationType::Bincode)
            .build();

        // gen some test data
        let test_data = gen_test_data();

        // compress and serialize
        let ser_compressed_data = data_parser
            .serialize_and_compress(&test_data)
            .await
            .unwrap();

        // deserialize and decompress
        let my_test_data_recreated = data_parser
            .decompress_and_deserialize::<MyTestData>(&ser_compressed_data)
            .await
            .unwrap();

        assert_eq!(my_test_data_recreated, test_data);
    }

    #[tokio::test]
    async fn test_nats_message_serialization() {
        // construct the data parser
        let data_parser = DataParserBuilder::new()
            .with_compression(CompressionType::Gzip)
            .with_compression_level(Level::Fastest)
            .with_serialization(SerializationType::Bincode)
            .build();

        // gen some test data
        let test_data = gen_test_data();

        // compress and serialize
        let nats_payload = data_parser
            .to_nats_payload(&test_data.parse(), &test_data)
            .await
            .unwrap();

        // deserialize and decompress
        let my_test_data_recreated = data_parser
            .from_nats_message::<MyTestData>(nats_payload)
            .await
            .unwrap();

        assert!(!my_test_data_recreated.subject.is_empty());
        assert!(!my_test_data_recreated.subject.is_empty());
        assert_eq!(my_test_data_recreated.data, test_data);
    }
}

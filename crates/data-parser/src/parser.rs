#![allow(dead_code)]

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
use tokio::io::AsyncWriteExt as _;

use crate::{
    error::{CompressionError, Error, SerdeError},
    types::{CompressionType, SerializationType},
};

/// Prost Message Wrapper allowing serialization/deserialization
pub(crate) struct ProstMessageSerdelizer<T: prost::Message>(pub(crate) T);

impl<T> ProstMessageSerdelizer<T>
where
    T: prost::Message + std::default::Default,
{
    /// Method to serialize
    pub(crate) fn serialize(&self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.0
            .encode(&mut buf)
            .map_err(|e| Error::Serde(SerdeError::ProstEncode(e)))?;
        Ok(buf)
    }

    /// Method to deserialize
    pub(crate) fn deserialize(buf: Vec<u8>) -> Result<T, Error> {
        T::decode(Bytes::from(buf))
            .map_err(|e| Error::Serde(SerdeError::ProstDecode(e)))
    }
}

macro_rules! define_compression_methods {
    ($($name:ident),+) => {
        paste::item! {
            $(
                pub(crate) async fn [<compress_ $name:lower>](&self, in_data: &[u8]) -> Result<Vec<u8>, Error> {
                    let mut encoder = [<$name Encoder>]::with_quality(Vec::new(), self.compression_level);
                    encoder.write_all(in_data).await.map_err(|e| Error::Compression(CompressionError::[<$name>](e)))?;
                    encoder.shutdown().await.map_err(|e| Error::Compression(CompressionError::[<$name>](e)))?;
                    Ok(encoder.into_inner())
                }

                pub(crate) async fn [<decompress_ $name:lower>](&self, in_data: &[u8]) -> Result<Vec<u8>, Error> {
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
#[derive(Debug)]
pub(crate) struct DataParser {
    pub compression_type: CompressionType,
    pub compression_level: Level,
    pub serialization_type: SerializationType,
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

    // Macro invocation to generate methods
    define_compression_methods!(Zlib, Gzip, Brotli, Bz, Lzma, Deflate, Zstd);

    /// Serializes and compresses the data
    pub(crate) async fn serialize_and_compress(
        &self,
        raw_data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let serialized_data = self.serialize(raw_data).await?;
        let compressed_data = self.compress(&serialized_data[..]).await?;
        Ok(compressed_data)
    }

    /// Decompresses and deserializes the data
    pub(crate) async fn decompress_and_deserialize(
        &self,
        raw_data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let decompressed_data = self.decompress(raw_data).await?;
        let deserialized_data =
            self.deserialize(&decompressed_data[..]).await?;
        Ok(deserialized_data)
    }

    /// Serializes the data
    pub(crate) async fn serialize(
        &self,
        raw_data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        match self.serialization_type {
            SerializationType::Bincode => bincode::serialize(raw_data)
                .map_err(|e| Error::Serde(SerdeError::Bincode(*e))),
            SerializationType::Postcard => postcard::to_allocvec(raw_data)
                .map_err(|e| Error::Serde(SerdeError::Postcard(e))),
        }
    }

    /// Deserializes the data
    pub(crate) async fn deserialize(
        &self,
        in_data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        match self.serialization_type {
            SerializationType::Bincode => bincode::deserialize(in_data)
                .map_err(|e| Error::Serde(SerdeError::Bincode(*e))),
            SerializationType::Postcard => postcard::from_bytes(in_data)
                .map_err(|e| Error::Serde(SerdeError::Postcard(e))),
        }
    }

    /// Compresses the data
    pub(crate) async fn compress(
        &self,
        raw_data: &[u8],
    ) -> Result<Vec<u8>, Error> {
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

    /// Decompresses the data
    pub(crate) async fn decompress(
        &self,
        raw_data: &[u8],
    ) -> Result<Vec<u8>, Error> {
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
}

#[cfg(test)]
mod test {
    use async_compression::Level;
    use rand::{thread_rng, Rng};
    use serde_json::Value;

    use crate::{
        builder::DataParserBuilder,
        types::{CompressionType, SerializationType},
    };

    #[tokio::test]
    async fn test_data_parser_builder() {
        let data_parser = DataParserBuilder::new()
            .with_compression(CompressionType::Gzip)
            .with_compression_level(Level::Fastest)
            .with_serialization(SerializationType::Bincode)
            .build();
    }
}

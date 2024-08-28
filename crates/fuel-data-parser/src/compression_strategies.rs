pub use async_compression::{
    tokio::write as compression_encoders_and_decoders,
    Level as CompressionLevel,
};
use tokio::io::AsyncWriteExt;

use crate::CompressionError;

mod private {
    pub trait Sealed {}
}

#[async_trait::async_trait]
pub trait CompressionStrategy: private::Sealed + Sync + Send {
    fn name(&self) -> &'static str;
    async fn compress(
        &self,
        uncompressed: &[u8],
    ) -> Result<Vec<u8>, CompressionError>;
    async fn decompress(
        &self,
        compressed: &[u8],
    ) -> Result<Vec<u8>, CompressionError>;
}

/// `name`: is the name of the compression strategy, usually a unit struct defined by you
/// `compression_type`: Valid inputs are ZLib, Gzip, Brotli, Bz, Lzma, Deflate, Zstd
/// `compression_level`: The compression level to use via the exported CompressionLevel enum
///
/// Example:
///
/// struct TestCompressionStrategy;
/// define_compression_strategy!(TestCompressionStrategy, Zlib, CompressionLevel::Fastest);
macro_rules! define_compression_strategy {
    ($name:ident, $compression_type:ident, $compression_level:ty) => {
        impl private::Sealed for $name {}

        #[async_trait::async_trait]
        impl CompressionStrategy for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }
            async fn compress(&self, uncompressed: &[u8]) -> Result<Vec<u8>, CompressionError> {
                paste::paste! {
                    let mut encoder = compression_encoders_and_decoders::[<$compression_type Encoder>]::with_quality(Vec::new(), $compression_level);
                    encoder
                        .write_all(uncompressed)
                        .await
                        .map_err(|e| CompressionError::[<$compression_type>](e))?;
                    encoder
                        .shutdown()
                        .await
                        .map_err(|e| CompressionError::[<$compression_type>](e))?;
                    Ok(encoder.into_inner())
                }
            }

            async fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>, CompressionError> {
                paste::paste! {
                    let mut decoder =  compression_encoders_and_decoders::[<$compression_type Decoder>]::new(Vec::new());
                    decoder
                        .write_all(compressed)
                        .await
                        .map_err(|e| CompressionError::[<$compression_type>](e))?;
                    decoder
                        .shutdown()
                        .await
                        .map_err(|e| CompressionError::[<$compression_type>](e))?;
                    Ok(decoder.into_inner())
                }

            }
        }
    };
}

#[derive(Clone)]
pub struct ZLibCompressionStrategy;
define_compression_strategy!(
    ZLibCompressionStrategy,
    Zlib,
    CompressionLevel::Fastest
);

#[cfg(feature = "bench-helpers")]
#[derive(Clone)]
pub struct GzipCompressionStrategy;
#[cfg(feature = "bench-helpers")]
define_compression_strategy!(
    GzipCompressionStrategy,
    Gzip,
    CompressionLevel::Fastest
);

#[cfg(feature = "bench-helpers")]
#[derive(Clone)]
pub struct BrotliCompressionStrategy;
#[cfg(feature = "bench-helpers")]
define_compression_strategy!(
    BrotliCompressionStrategy,
    Brotli,
    CompressionLevel::Fastest
);

#[cfg(feature = "bench-helpers")]
#[derive(Clone)]
pub struct BzCompressionStrategy;
#[cfg(feature = "bench-helpers")]
define_compression_strategy!(
    BzCompressionStrategy,
    Bz,
    CompressionLevel::Fastest
);

#[cfg(feature = "bench-helpers")]
#[derive(Clone)]
pub struct LzmaCompressionStrategy;
#[cfg(feature = "bench-helpers")]
define_compression_strategy!(
    LzmaCompressionStrategy,
    Lzma,
    CompressionLevel::Fastest
);

#[cfg(feature = "bench-helpers")]
#[derive(Clone)]
pub struct DeflateCompressionStrategy;
#[cfg(feature = "bench-helpers")]
define_compression_strategy!(
    DeflateCompressionStrategy,
    Deflate,
    CompressionLevel::Fastest
);

#[cfg(feature = "bench-helpers")]
#[derive(Clone)]
pub struct ZstdCompressionStrategy;
#[cfg(feature = "bench-helpers")]
define_compression_strategy!(
    ZstdCompressionStrategy,
    Zstd,
    CompressionLevel::Fastest
);

use std::sync::Arc;

lazy_static::lazy_static! {
    pub static ref DEFAULT_COMPRESSION_STRATEGY: Arc<dyn CompressionStrategy> =  Arc::new(ZLibCompressionStrategy);
}

#[cfg(feature = "bench-helpers")]
lazy_static::lazy_static! {
    pub static ref ALL_COMPRESSION_STRATEGIES: [Arc<dyn CompressionStrategy>; 7] = [
        Arc::new(ZLibCompressionStrategy),
        Arc::new(GzipCompressionStrategy),
        Arc::new(BrotliCompressionStrategy),
        Arc::new(BzCompressionStrategy),
        Arc::new(LzmaCompressionStrategy),
        Arc::new(DeflateCompressionStrategy),
        Arc::new(ZstdCompressionStrategy),
    ];

}

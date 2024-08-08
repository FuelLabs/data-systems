mod builder;
mod error;
mod parser;
mod types;
pub use async_compression::Level;

pub use crate::{
    builder::DataParserBuilder,
    error::{CompressionError, Error, SerdeError},
    parser::{DataParser, ProstDataParser},
    types::*,
    Level as CompressionLevel,
};

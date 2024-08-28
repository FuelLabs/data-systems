#![doc = include_str!("../README.md")]

pub mod blocks;
pub mod nats;
pub mod stream;
pub mod transactions;
pub mod types;

mod stream_encoding;

pub use stream::*;

pub mod prelude {
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        blocks::subjects::*,
        nats::*,
        stream::*,
        stream_encoding::*,
        transactions::subjects::*,
        types::*,
    };

    #[cfg(feature = "test-helpers")]
    pub static NATS_URL: &str = "localhost:4222";
}

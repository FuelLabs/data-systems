#![doc = include_str!("../README.md")]

pub mod blocks;
pub mod inputs;
pub mod nats;
pub mod stream;
pub mod transactions;
pub mod types;

pub use stream::*;

pub mod prelude {
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        blocks::subjects::*,
        inputs::subjects::*,
        nats::*,
        stream::*,
        transactions::subjects::*,
        types::*,
    };

    #[cfg(feature = "test-helpers")]
    pub static NATS_URL: &str = "localhost:4222";
}

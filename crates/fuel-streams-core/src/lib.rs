#![doc = include_str!("../README.md")]

pub mod blocks;
pub mod inputs;
pub mod logs;
pub mod nats;
pub mod outputs;
pub mod receipts;
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
        outputs::subjects::*,
        stream::*,
        transactions::subjects::*,
        types::*,
    };

    #[cfg(feature = "test-helpers")]
    pub static NATS_URL: &str = "localhost:4222";
}

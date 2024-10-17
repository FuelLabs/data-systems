#![doc = include_str!("../README.md")]

pub mod blocks;
pub mod inputs;
pub mod logs;
pub mod outputs;
pub mod receipts;
pub mod transactions;
pub mod utxos;

pub mod nats;
pub mod stream;
pub mod types;

pub use stream::*;

pub mod subjects {
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        blocks::subjects::*,
        inputs::subjects::*,
        logs::subjects::*,
        outputs::subjects::*,
        receipts::subjects::*,
        transactions::subjects::*,
        utxos::subjects::*,
    };
}

pub mod prelude {
    pub use fuel_streams_macros::subject::*;

    pub use crate::{nats::*, stream::*, subjects::*, types::*};

    #[cfg(feature = "test-helpers")]
    pub static NATS_URL: &str = "localhost:4222";
}

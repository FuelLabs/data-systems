#![doc = include_str!("../README.md")]

pub mod blocks;
pub mod executors;
pub mod inputs;
pub mod logs;
pub mod outputs;
pub mod receipts;
pub mod transactions;
pub mod utxos;

pub mod nats;
pub mod stream;

pub mod subjects;

pub mod fuel_core_like;
mod fuel_core_types;
mod primitive_types;
pub mod types;

pub use stream::*;

pub mod prelude {
    #[allow(unused_imports)]
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        executors::*,
        fuel_core_like::*,
        nats::*,
        stream::*,
        subjects::*,
        types::*,
    };
}

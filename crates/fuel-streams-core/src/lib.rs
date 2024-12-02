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

pub mod subjects;

mod fuel_core_types;
mod primitive_types;
pub mod types;

pub use stream::*;

pub mod prelude {
    pub use fuel_streams_macros::subject::*;

    pub use crate::{nats::*, stream::*, subjects::*, types::*};
}

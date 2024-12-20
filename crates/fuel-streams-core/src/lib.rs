#![doc = include_str!("../README.md")]

pub mod blocks;
pub mod inputs;
pub mod logs;
pub mod outputs;
pub mod receipts;
pub mod transactions;
pub mod utxos;

pub mod nats {
    pub use fuel_streams_nats::*;
}

pub mod s3 {
    pub use fuel_streams_storage::s3::*;
}

pub mod stream;

pub mod subjects;

pub mod fuel_core_like;
mod fuel_core_types;
mod primitive_types;
pub mod types;

pub use stream::*;

pub mod prelude {
    pub use fuel_networks::*;
    #[allow(unused_imports)]
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        fuel_core_like::*,
        nats::*,
        s3::*,
        stream::*,
        subjects::*,
        types::*,
    };
}

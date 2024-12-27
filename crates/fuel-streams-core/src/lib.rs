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

pub mod storage {
    pub use fuel_streams_storage::*;
}

pub(crate) mod data_parser {
    pub use fuel_data_parser::*;
}

pub mod stream;
pub mod subjects;

pub mod fuel_core_like;
mod fuel_core_types;
mod primitive_types;
pub mod types;

pub(crate) use data_parser::*;
pub use stream::*;

pub mod prelude {
    #[allow(unused_imports)]
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        data_parser::*,
        fuel_core_like::*,
        nats::*,
        storage::*,
        stream::*,
        subjects::*,
        types::*,
    };
}

#![doc = include_str!("../README.md")]

pub mod fuel_core_like;
pub mod stream;
pub mod subjects;

pub use fuel_core_like::*;
pub use fuel_streams_nats as nats;
pub use fuel_streams_types as types;
pub use stream::*;

pub mod prelude {
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        fuel_core_like::*,
        nats::*,
        stream::*,
        subjects::*,
        types::*,
    };
}

macro_rules! export_module {
    ($module:ident) => {
        pub mod $module {
            pub use crate::subjects::$module::*;
        }
    };
}

export_module!(blocks);
export_module!(inputs);
export_module!(logs);
export_module!(outputs);
export_module!(receipts);
export_module!(transactions);
export_module!(utxos);

#![doc = include_str!("../README.md")]

pub mod fuel_core_like;
pub mod stream;

pub use fuel_core_like::*;
pub use fuel_streams_nats as nats;
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

pub mod types {
    pub use fuel_streams_domains::{
        blocks::types::*,
        inputs::types::*,
        outputs::types::*,
        receipts::types::*,
        transactions::types::*,
        utxos::types::*,
    };
    pub use fuel_streams_types::*;
}

pub mod subjects {
    pub use fuel_streams_domains::{
        blocks::subjects::*,
        inputs::subjects::*,
        outputs::subjects::*,
        receipts::subjects::*,
        transactions::subjects::*,
        utxos::subjects::*,
    };
    pub use fuel_streams_macros::subject::*;
}

macro_rules! export_module {
    ($module:ident) => {
        pub mod $module {
            pub use fuel_streams_domains::$module::subjects::*;
        }
    };
}

export_module!(blocks);
export_module!(inputs);
export_module!(outputs);
export_module!(receipts);
export_module!(transactions);
export_module!(utxos);

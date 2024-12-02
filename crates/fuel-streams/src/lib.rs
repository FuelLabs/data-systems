#![doc = include_str!("../README.md")]

pub mod client;
pub mod error;
pub mod stream;

pub use error::*;
pub use stream::*;

pub mod subjects {
    pub use fuel_streams_core::subjects::*;
}

pub mod types {
    pub use fuel_streams_core::{
        nats::{types::*, FuelNetwork, NatsClientOpts},
        types::*,
    };

    pub use crate::client::types::*;
}

macro_rules! export_module {
    ($module:ident, $($submodule:ident),+) => {
        pub mod $module {
            $(
                pub use fuel_streams_core::$module::$submodule::*;
            )+
        }
    };
}

export_module!(blocks, subjects, types);
export_module!(inputs, subjects, types);
export_module!(logs, subjects, types);
export_module!(outputs, subjects, types);
export_module!(receipts, subjects);
export_module!(transactions, subjects, types);
export_module!(utxos, subjects, types);

#[cfg(any(test, feature = "test-helpers"))]
pub mod prelude {
    pub use crate::{client::*, error::*, stream::*, types::*};
}

#![doc = include_str!("../README.md")]

pub mod server;
pub mod stream;

pub use stream::*;

pub mod prelude {
    pub use fuel_streams_subject::subject::*;

    pub use crate::{stream::*, subjects::*, types::*};
}

pub mod types {
    pub use fuel_streams_domains::{
        blocks::types::*,
        inputs::types::*,
        messages::types::*,
        outputs::types::*,
        predicates::types::*,
        receipts::types::*,
        transactions::types::*,
        utxos::types::*,
    };
    pub use fuel_streams_types::*;

    pub use crate::server::*;
}

pub mod subjects {
    pub use fuel_streams_domains::{
        blocks::subjects::*,
        inputs::subjects::*,
        messages::subjects::*,
        outputs::subjects::*,
        predicates::subjects::*,
        receipts::subjects::*,
        transactions::subjects::*,
        utxos::subjects::*,
    };
    pub use fuel_streams_subject::subject::*;
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
export_module!(messages);
export_module!(outputs);
export_module!(predicates);
export_module!(receipts);
export_module!(transactions);
export_module!(utxos);

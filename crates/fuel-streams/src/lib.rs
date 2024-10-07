#![doc = include_str!("../README.md")]

pub mod client;
pub mod error;
pub mod stream;

pub use error::*;
pub use stream::*;

pub mod core {
    pub use fuel_streams_core::*;
}

pub mod types {
    pub use fuel_streams_core::{nats::NatsClientOpts, types::*};

    pub use crate::client::types::*;
}

pub mod transactions {
    pub use fuel_streams_core::transactions::{subjects::*, types::*};
}

pub mod blocks {
    pub use fuel_streams_core::blocks::{subjects::*, types::*};
}

pub mod receipts {
    pub use fuel_streams_core::receipts::subjects::*;
}

#[cfg(any(test, feature = "test-helpers"))]
pub mod prelude {
    pub use crate::{client::*, error::*, stream::*, types::*};
}

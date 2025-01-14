#![doc = include_str!("../README.md")]

pub mod client;
pub mod error;
pub mod networks;

pub use client::*;
pub use error::*;
pub use networks::*;

pub mod subjects {
    pub use fuel_streams_core::subjects::*;
}

pub mod types {
    pub use fuel_streams_core::types::*;

    pub use crate::client::types::*;
}

#[cfg(any(test, feature = "test-helpers"))]
pub mod prelude {
    pub use crate::{client::*, error::*, networks::*, subjects::*, types::*};
}

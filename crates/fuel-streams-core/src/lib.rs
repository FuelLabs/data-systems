pub mod blocks;
pub mod nats;
pub mod stream;
pub mod transactions;
pub mod types;

pub use stream::{Stream, StreamError};

pub mod prelude {
    pub use fuel_streams_macros::subject::*;

    pub use crate::{
        blocks::subjects::*,
        nats::*,
        stream::*,
        transactions::subjects::*,
        types::*,
    };

    #[cfg(feature = "test-helpers")]
    pub static NATS_URL: &str = "localhost:4222";
}

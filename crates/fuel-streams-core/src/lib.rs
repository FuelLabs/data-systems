pub mod blocks;
pub mod nats;
mod stream;
pub mod transactions;
pub mod types;

pub use stream::*;

pub mod prelude {
    pub use crate::{
        blocks::subjects::*,
        nats::*,
        transactions::subjects::*,
        types::*,
    };

    #[cfg(feature = "test-helpers")]
    pub static NATS_URL: &str = "localhost:4222";
}

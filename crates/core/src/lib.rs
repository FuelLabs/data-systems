pub mod nats;
pub mod types;

pub mod prelude {
    pub use crate::{nats::*, types::*};

    #[cfg(feature = "test-helpers")]
    pub static NATS_URL: &str = "localhost:4222";
}

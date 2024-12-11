use std::error::Error;

pub use crate::{
    blocks::types::*,
    fuel_core_types::*,
    inputs::types::*,
    logs::types::*,
    nats::types::*,
    outputs::types::*,
    primitive_types::*,
    receipts::types::*,
    transactions::types::*,
    utxos::types::*,
};

// ------------------------------------------------------------------------
// General
// ------------------------------------------------------------------------
pub type BoxedError = Box<dyn Error + Send + Sync>;
pub type BoxedResult<T> = Result<T, BoxedError>;

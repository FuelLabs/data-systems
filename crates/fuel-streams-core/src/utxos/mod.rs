pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{DataEncoder, StreamError, Streamable};

impl DataEncoder for Utxo {
    type Err = StreamError;
}
impl Streamable for Utxo {
    const NAME: &'static str = "utxos";
    const WILDCARD_LIST: &'static [&'static str] = &[UtxosSubject::WILDCARD];
}

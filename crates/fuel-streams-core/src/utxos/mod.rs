pub mod subjects;
pub mod types;

use serde::{Deserialize, Serialize};
pub use subjects::*;

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo(pub Option<Vec<u8>>);

impl StreamEncoder for Utxo {}
impl Streamable for Utxo {
    const NAME: &'static str = "utxos";
    const WILDCARD_LIST: &'static [&'static str] = &[UtxosSubject::WILDCARD];
}

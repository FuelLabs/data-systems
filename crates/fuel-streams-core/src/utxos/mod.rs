pub mod subjects;
pub mod types;
pub use subjects::*;
use types::Utxo;

use crate::prelude::*;

impl StreamEncoder for Utxo {}
impl Streamable for Utxo {
    const NAME: &'static str = "utxos";
    const WILDCARD_LIST: &'static [&'static str] = &[UtxosSubject::WILDCARD];
}

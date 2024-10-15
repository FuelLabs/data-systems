pub mod subjects;
pub mod types;

pub use subjects::*;
use types::*;

use crate::{StreamEncoder, Streamable};

impl StreamEncoder for Receipt {}
impl Streamable for Receipt {
    const NAME: &'static str = "receipts";
    const WILDCARD_LIST: &'static [&'static str] =
        &[ReceiptsCallSubject::WILDCARD, ReceiptsByIdSubject::WILDCARD];
}

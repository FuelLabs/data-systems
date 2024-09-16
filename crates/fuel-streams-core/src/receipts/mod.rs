pub mod subjects;

pub use subjects::*;

use crate::prelude::*;

impl StreamEncoder for Receipt {}
impl Streamable for Receipt {
    const NAME: &'static str = "receipts";
    const WILDCARD_LIST: &'static [&'static str] =
        &[ReceiptsCallSubject::WILDCARD, ReceiptsByIdSubject::WILDCARD];
}

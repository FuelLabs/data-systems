pub mod subjects;
mod transaction_ext;
pub mod types;

pub use subjects::*;
pub use transaction_ext::*;
use types::*;

use crate::{StreamEncoder, Streamable};

impl StreamEncoder for Transaction {}
impl Streamable for Transaction {
    const NAME: &'static str = "transactions";
    const WILDCARD_LIST: &'static [&'static str] = &[
        TransactionsSubject::WILDCARD,
        TransactionsByIdSubject::WILDCARD,
    ];
}

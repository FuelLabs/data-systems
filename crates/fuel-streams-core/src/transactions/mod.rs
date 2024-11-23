pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{StreamEncoder, Streamable};

impl StreamEncoder for Transaction {}
impl Streamable for Transaction {
    const NAME: &'static str = "transactions";
    const WILDCARD_LIST: &'static [&'static str] = &[
        TransactionsSubject::WILDCARD,
        TransactionsByIdSubject::WILDCARD,
    ];
}

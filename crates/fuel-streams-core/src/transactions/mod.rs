pub mod subjects;
pub mod types;

use fuel_streams_macros::subject::IntoSubject;
pub use subjects::*;
use types::*;

use crate::prelude::*;

impl StreamEncoder for Transaction {}

impl Streamable for Transaction {
    const NAME: &'static str = "transactions";
    const WILDCARD_LIST: &'static [&'static str] = &[
        TransactionsSubject::WILDCARD,
        TransactionsByIdSubject::WILDCARD,
    ];
}

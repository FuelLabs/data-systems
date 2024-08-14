pub mod subjects;
pub mod types;

use fuel_streams_macros::subject::IntoSubject;
pub use subjects::*;
use types::*;

use crate::{
    nats::stream::{NatsStore, Streamable},
    prelude::StreamEncoder,
};

impl StreamEncoder for Transaction {}
impl Streamable for Transaction {
    const NAME: &'static str = "transactions";
    const WILDCARD_LIST: &'static [&'static str] = &[
        TransactionsSubject::WILDCARD,
        TransactionsByIdSubject::WILDCARD,
    ];

    type Builder = NatsStore<Self>;
    type MainSubject = TransactionsSubject;
}

pub mod subjects;
pub mod types;

use fuel_core_types::fuel_tx::{field::Outputs, Output};
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

pub trait WithTxInputs {
    fn inputs(&self) -> &[Input];
}

pub trait WithTxOutputs {
    fn outputs(&self) -> &Vec<Output>;
}

impl WithTxInputs for Transaction {
    fn inputs(&self) -> &[Input] {
        match self {
            Transaction::Mint(_) => &[],
            Transaction::Script(tx) => tx.inputs(),
            Transaction::Blob(tx) => tx.inputs(),
            Transaction::Create(tx) => tx.inputs(),
            Transaction::Upload(tx) => tx.inputs(),
            Transaction::Upgrade(tx) => tx.inputs(),
        }
    }
}

impl WithTxOutputs for Transaction {
    fn outputs(&self) -> &Vec<Output> {
        match self {
            Transaction::Mint(_) => {
                static NO_OUTPUTS: Vec<Output> = Vec::new();
                &NO_OUTPUTS
            }
            Transaction::Script(tx) => tx.outputs(),
            Transaction::Blob(tx) => tx.outputs(),
            Transaction::Create(tx) => tx.outputs(),
            Transaction::Upload(tx) => tx.outputs(),
            Transaction::Upgrade(tx) => tx.outputs(),
        }
    }
}

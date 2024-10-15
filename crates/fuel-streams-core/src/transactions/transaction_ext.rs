use crate::types::*;

pub trait TransactionExt {
    fn inputs(&self) -> &[Input];
    fn outputs(&self) -> &Vec<Output>;
}

impl TransactionExt for Transaction {
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

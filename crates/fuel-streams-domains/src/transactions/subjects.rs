use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use crate::{blocks::types::*, transactions::types::*};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "transactions")]
#[subject(wildcard = "transactions.>")]
#[subject(
    format = "transactions.{block_height}.{tx_id}.{tx_index}.{tx_status}.{kind}"
)]
pub struct TransactionsSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub tx_status: Option<TransactionStatus>,
    pub kind: Option<TransactionKind>,
}

impl From<&Transaction> for TransactionsSubject {
    fn from(transaction: &Transaction) -> Self {
        let subject = TransactionsSubject::new();
        subject
            .with_tx_id(Some(transaction.id.clone()))
            .with_kind(Some(transaction.kind.clone()))
    }
}

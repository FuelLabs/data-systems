use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::TransactionsQuery;
use crate::transactions::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "transactions")]
#[subject(entity = "Transaction")]
#[subject(query_all = "transactions.>")]
#[subject(
    format = "transactions.{block_height}.{tx_id}.{tx_index}.{tx_status}.{tx_type}"
)]
pub struct TransactionsSubject {
    #[subject(
        description = "The height of the block containing this transaction"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(
        description = "The status of the transaction (success, failure, or submitted)"
    )]
    pub tx_status: Option<TransactionStatus>,
    #[subject(description = "The type of transaction (create, mint, script)")]
    #[subject(sql_column = "type")]
    pub tx_type: Option<TransactionType>,
}

impl From<&Transaction> for TransactionsSubject {
    fn from(transaction: &Transaction) -> Self {
        let subject = TransactionsSubject::new();
        subject
            .with_tx_id(Some(transaction.id.clone()))
            .with_tx_type(Some(transaction.r#type))
    }
}

impl From<TransactionsSubject> for TransactionsQuery {
    fn from(subject: TransactionsSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            tx_status: subject.tx_status,
            r#type: subject.tx_type,
            ..Default::default()
        }
    }
}

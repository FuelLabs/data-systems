use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::{
    infra::{record::QueryOptions, repository::SubjectQueryBuilder},
    transactions::types::*,
};

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
    pub tx_index: Option<u32>,
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
            .with_tx_type(Some(transaction.tx_type.clone()))
    }
}

impl SubjectQueryBuilder for TransactionsSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        query_builder.push("SELECT * FROM transactions");

        if let Some(where_clause) = self.to_sql_where() {
            conditions.push(where_clause);
        }
        if let Some(block) = options.map(|o| o.from_block.unwrap_or_default()) {
            conditions.push(format!("block_height >= {}", block));
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        query_builder.push(" ORDER BY block_height ASC, tx_index ASC");
        if let Some(opts) = options {
            opts.apply_limit_offset(&mut query_builder);
        }

        query_builder
    }
}

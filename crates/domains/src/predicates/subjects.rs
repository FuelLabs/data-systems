use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::infra::{record::QueryOptions, repository::SubjectQueryBuilder};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "predicates")]
#[subject(entity = "Predicate")]
#[subject(query_all = "predicates.>")]
#[subject(
    format = "predicates.{block_height}.{tx_id}.{tx_index}.{input_index}.{blob_id}.{predicate_address}"
)]
pub struct PredicatesSubject {
    #[subject(description = "The height of the block containing this UTXO")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this predicate (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this input within the transaction that had this predicate"
    )]
    pub input_index: Option<u32>,
    #[subject(
        description = "The ID of the blob containing the predicate bytecode"
    )]
    pub blob_id: Option<HexData>,
    #[subject(
        description = "The address of the predicate (32 byte string prefixed by 0x)"
    )]
    pub predicate_address: Option<Address>,
}

impl SubjectQueryBuilder for PredicatesSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();

        // Join predicates with predicate_transactions to get all fields
        query_builder.push(
            "SELECT p.*, pt.block_height, pt.tx_id, pt.tx_index, pt.input_index, pt.subject
             FROM predicates p
             JOIN predicate_transactions pt ON p.id = pt.predicate_id"
        );

        if let Some(where_clause) = self.to_sql_where() {
            conditions.push(where_clause);
        }
        if let Some(block) = options.map(|o| o.from_block.unwrap_or_default()) {
            conditions.push(format!("pt.block_height >= {}", block));
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        query_builder.push(" ORDER BY pt.block_height ASC, pt.tx_index ASC, pt.input_index ASC");
        if let Some(opts) = options {
            opts.apply_limit_offset(&mut query_builder);
        }

        query_builder
    }
}

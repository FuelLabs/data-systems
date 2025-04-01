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
    format = "predicates.{block_height}.{tx_id}.{tx_index}.{input_index}.{blob_id}.{predicate_address}.{asset}"
)]
pub struct PredicatesSubject {
    #[subject(
        description = "The height of the block containing this predicate"
    )]
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
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the coin (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

impl PredicatesSubject {
    pub fn to_sql_where(&self) -> Option<String> {
        let mut conditions = Vec::new();

        if let Some(block_height) = self.block_height {
            conditions.push(format!("pt.block_height = '{}'", block_height));
        }
        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("pt.tx_id = '{}'", tx_id));
        }
        if let Some(tx_index) = self.tx_index {
            conditions.push(format!("pt.tx_index = '{}'", tx_index));
        }
        if let Some(input_index) = self.input_index {
            conditions.push(format!("pt.input_index = '{}'", input_index));
        }
        if let Some(blob_id) = &self.blob_id {
            conditions.push(format!("p.blob_id = '{}'", blob_id));
        }
        if let Some(predicate_address) = &self.predicate_address {
            conditions
                .push(format!("p.predicate_address = '{}'", predicate_address));
        }
        if let Some(asset) = &self.asset {
            conditions.push(format!("pt.asset_id = '{}'", asset));
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }
}

impl SubjectQueryBuilder for PredicatesSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();

        query_builder.push(
            "SELECT p.id, p.blob_id, p.predicate_address, p.created_at, p.published_at,
                    pt.subject, pt.block_height, pt.tx_id, pt.tx_index, pt.input_index,
                    pt.asset_id, pt.bytecode
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predicates_subject_sql_where() {
        let tx_id = TxId::random();
        let predicate_address = Address::random();
        let subject = PredicatesSubject {
            block_height: Some(123.into()),
            tx_id: Some(tx_id.clone()),
            predicate_address: Some(predicate_address.clone()),
            ..Default::default()
        };

        let expected = Some(format!(
            "pt.block_height = '123' AND pt.tx_id = '{}' AND p.predicate_address = '{}'",
            tx_id, predicate_address
        ));
        assert_eq!(subject.to_sql_where(), expected);
    }
}

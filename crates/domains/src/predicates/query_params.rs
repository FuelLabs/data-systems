use fuel_streams_types::{Address, BlockHeight, TxId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::infra::repository::{
    HasPagination,
    QueryPagination,
    QueryParamsBuilder,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct PredicatesQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<i32>,
    pub block_height: Option<BlockHeight>,
    pub blob_id: Option<String>,
    pub predicate_address: Option<Address>,
    #[serde(flatten)]
    pub pagination: QueryPagination,
}

impl QueryParamsBuilder for PredicatesQuery {
    fn query_builder(&self) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        query_builder.push("SELECT * FROM predicates");

        if let Some(tx_id) = &self.tx_id {
            conditions.push("tx_id = ");
            query_builder.push_bind(tx_id.to_string());
            query_builder.push(" ");
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push("tx_index = ");
            query_builder.push_bind(*tx_index as i32);
            query_builder.push(" ");
        }

        if let Some(input_index) = &self.input_index {
            conditions.push("input_index = ");
            query_builder.push_bind(*input_index);
            query_builder.push(" ");
        }

        if let Some(block_height) = &self.block_height {
            conditions.push("block_height = ");
            query_builder.push_bind(*block_height);
            query_builder.push(" ");
        }

        if let Some(blob_id) = &self.blob_id {
            conditions.push("blob_id = ");
            query_builder.push_bind(blob_id.clone());
            query_builder.push(" ");
        }

        if let Some(predicate_address) = &self.predicate_address {
            conditions.push("predicate_address = ");
            query_builder.push_bind(predicate_address.to_string());
            query_builder.push(" ");
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        // Apply pagination using block_height as cursor
        self.pagination
            .apply_pagination(&mut query_builder, "block_height");

        query_builder
    }
}

impl HasPagination for PredicatesQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

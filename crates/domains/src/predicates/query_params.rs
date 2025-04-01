use fuel_streams_types::{Address, AssetId, BlockHeight, HexData, TxId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PredicatesQuery {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<i32>,
    pub blob_id: Option<HexData>,
    pub predicate_address: Option<Address>,
    pub asset: Option<AssetId>,
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
}

impl QueryParamsBuilder for PredicatesQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }

    fn pagination_mut(&mut self) -> &mut QueryPagination {
        &mut self.pagination
    }

    fn with_pagination(&mut self, pagination: &QueryPagination) {
        self.pagination = pagination.clone();
    }

    fn options(&self) -> &QueryOptions {
        &self.options
    }

    fn options_mut(&mut self) -> &mut QueryOptions {
        &mut self.options
    }

    fn with_options(&mut self, options: &QueryOptions) {
        self.options = options.clone();
    }

    fn query_builder(&self) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();

        query_builder.push(
            "SELECT p.*, pt.subject, pt.block_height, pt.tx_id, pt.tx_index, pt.input_index, pt.asset_id, pt.bytecode
             FROM predicates p
             JOIN predicate_transactions pt ON p.id = pt.predicate_id",
        );

        if let Some(block_height) = &self.block_height {
            conditions.push("pt.block_height = ");
            query_builder.push_bind(*block_height);
            query_builder.push(" ");
        }

        if let Some(tx_id) = &self.tx_id {
            conditions.push("pt.tx_id = ");
            query_builder.push_bind(tx_id.to_string());
            query_builder.push(" ");
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push("pt.tx_index = ");
            query_builder.push_bind(*tx_index as i32);
            query_builder.push(" ");
        }

        if let Some(input_index) = &self.input_index {
            conditions.push("pt.input_index = ");
            query_builder.push_bind(*input_index);
            query_builder.push(" ");
        }

        if let Some(blob_id) = &self.blob_id {
            conditions.push("p.blob_id = ");
            query_builder.push_bind(blob_id.to_string());
            query_builder.push(" ");
        }

        if let Some(predicate_address) = &self.predicate_address {
            conditions.push("p.predicate_address = ");
            query_builder.push_bind(predicate_address.to_string());
            query_builder.push(" ");
        }

        if let Some(asset) = &self.asset {
            conditions.push("pt.asset_id = ");
            query_builder.push_bind(asset.to_string());
            query_builder.push(" ");
        }

        let options = &self.options;
        if let Some(from_block) = options.from_block {
            conditions.push("pt.block_height >= ");
            query_builder.push_bind(from_block);
            query_builder.push(" ");
        }
        #[cfg(any(test, feature = "test-helpers"))]
        if let Some(ns) = &options.namespace {
            conditions.push("pt.subject LIKE ");
            query_builder.push_bind(format!("{}%", ns));
            query_builder.push(" ");
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        self.pagination
            .apply_on_query(&mut query_builder, "pt.block_height");

        query_builder
    }
}

impl HasPagination for PredicatesQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

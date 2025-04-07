use fuel_streams_types::{Address, AssetId, BlockHeight, HexData, TxId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct PredicatesQuery {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<i32>,
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
            conditions.push(format!("pt.block_height = {}", block_height));
        }

        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("pt.tx_id = '{}'", tx_id));
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push(format!("pt.tx_index = {}", tx_index));
        }

        if let Some(input_index) = &self.input_index {
            conditions.push(format!("pt.input_index = {}", input_index));
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

        Self::apply_conditions(
            &mut query_builder,
            &mut conditions,
            &self.options,
            &self.pagination,
            "cursor",
            Some("pt."),
        );

        Self::apply_pagination(
            &mut query_builder,
            &self.pagination,
            "cursor",
            Some("pt."),
        );

        query_builder
    }
}

impl HasPagination for PredicatesQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

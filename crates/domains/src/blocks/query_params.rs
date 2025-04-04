use fuel_streams_types::{Address, BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::Block;
use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
    TimeRange,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct BlocksQuery {
    pub producer: Option<Address>,
    pub height: Option<BlockHeight>,
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
}

impl From<&Block> for BlocksQuery {
    fn from(block: &Block) -> Self {
        let mut options = QueryOptions::default();
        options.with_time_range(Some(TimeRange::All));
        options.with_timestamp(Some(BlockTimestamp::from(&block.header)));
        Self {
            options,
            producer: Some(block.producer.clone()),
            height: Some(block.height),
            ..Default::default()
        }
    }
}

impl QueryParamsBuilder for BlocksQuery {
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
        query_builder.push("SELECT * FROM blocks");

        if let Some(producer) = &self.producer {
            conditions.push(format!("producer_address = '{}'", producer));
        }

        if let Some(height) = &self.height {
            conditions.push(format!("block_height = {}", height));
        }

        Self::apply_conditions(
            &mut query_builder,
            &mut conditions,
            &self.options,
            &self.pagination,
            "block_height",
            None,
        );

        Self::apply_pagination(
            &mut query_builder,
            &self.pagination,
            "block_height",
            None,
        );

        query_builder
    }
}

impl HasPagination for BlocksQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

use fuel_streams_types::{Address, BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::{time_range::TimeRange, Block};
use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlocksQuery {
    pub producer: Option<Address>,
    pub height: Option<BlockHeight>,
    pub timestamp: Option<BlockTimestamp>,
    pub time_range: Option<TimeRange>,
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
}

impl From<&Block> for BlocksQuery {
    fn from(block: &Block) -> Self {
        Self {
            producer: Some(block.producer.clone()),
            height: Some(block.height),
            timestamp: Some(BlockTimestamp::from(&block.header)),
            time_range: Some(TimeRange::All),
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
            conditions.push("producer_address = ");
            query_builder.push_bind(producer.to_string());
            query_builder.push(" ");
        }

        if let Some(height) = &self.height {
            conditions.push("block_height = ");
            query_builder.push_bind(*height);
            query_builder.push(" ");
        }

        if let Some(timestamp) = &self.timestamp {
            conditions.push("created_at >= ");
            query_builder.push_bind(timestamp.unix_timestamp());
            query_builder.push(" ");
        }

        if let Some(time_range) = &self.time_range {
            let start_time = time_range.time_since_now();
            conditions.push("created_at >= ");
            query_builder.push_bind(start_time.timestamp());
            query_builder.push(" ");
        }

        let options = &self.options;
        if let Some(from_block) = options.from_block {
            conditions.push("block_height >= ");
            query_builder.push_bind(from_block);
            query_builder.push(" ");
        }
        #[cfg(any(test, feature = "test-helpers"))]
        if let Some(ns) = &options.namespace {
            conditions.push("subject LIKE ");
            query_builder.push_bind(format!("{}%", ns));
            query_builder.push(" ");
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        self.pagination
            .apply_on_query(&mut query_builder, "block_height");

        query_builder
    }
}

impl HasPagination for BlocksQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

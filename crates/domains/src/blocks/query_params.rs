use fuel_streams_types::{Address, BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::{time_range::TimeRange, Block};
use crate::infra::repository::{
    HasPagination,
    QueryPagination,
    QueryParamsBuilder,
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
}

impl From<&Block> for BlocksQuery {
    fn from(block: &Block) -> Self {
        Self {
            producer: Some(block.producer.clone()),
            height: Some(block.height),
            timestamp: Some(BlockTimestamp::from(&block.header)),
            time_range: Some(TimeRange::All),
            pagination: QueryPagination::default(),
        }
    }
}

impl QueryParamsBuilder for BlocksQuery {
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

impl HasPagination for BlocksQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

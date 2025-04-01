use fuel_streams_types::BlockHeight;
use sqlx::{Postgres, QueryBuilder};

use super::{QueryPagination, DEFAULT_LIMIT};
use crate::infra::repository::QueryOptions;

pub trait QueryParamsBuilder {
    fn query_builder(&self) -> QueryBuilder<'static, Postgres>;

    fn pagination(&self) -> &QueryPagination;
    fn pagination_mut(&mut self) -> &mut QueryPagination;
    fn with_pagination(&mut self, pagination: &QueryPagination);

    fn options(&self) -> &QueryOptions;
    fn options_mut(&mut self) -> &mut QueryOptions;
    fn with_options(&mut self, options: &QueryOptions);

    fn with_from_block(
        &mut self,
        from_block: Option<BlockHeight>,
    ) -> &mut Self {
        self.options_mut().from_block = from_block;
        self
    }

    fn with_namespace(&mut self, namespace: Option<String>) -> &mut Self {
        self.options_mut().namespace = namespace;
        self
    }

    fn with_offset(&mut self, offset: Option<i32>) -> &mut Self {
        self.pagination_mut().offset = offset;
        self
    }

    fn with_limit(&mut self, limit: Option<i32>) -> &mut Self {
        self.pagination_mut().limit = limit;
        self
    }

    fn with_after(&mut self, after: Option<i32>) -> &mut Self {
        self.pagination_mut().after = after;
        self
    }

    fn with_before(&mut self, before: Option<i32>) -> &mut Self {
        self.pagination_mut().before = before;
        self
    }

    fn with_first(&mut self, first: Option<i32>) -> &mut Self {
        self.pagination_mut().first = first;
        self
    }

    fn with_last(&mut self, last: Option<i32>) -> &mut Self {
        self.pagination_mut().last = last;
        self
    }

    fn increment_offset(&mut self) -> &mut Self {
        if let Some(offset) = self.pagination_mut().offset {
            let limit = self.pagination().limit.unwrap_or(DEFAULT_LIMIT);
            self.pagination_mut().offset = Some(offset + limit);
        }
        self
    }
}

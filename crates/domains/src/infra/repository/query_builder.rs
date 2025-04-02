use fuel_streams_types::BlockHeight;
use sqlx::{Postgres, QueryBuilder};

use super::{OrderBy, QueryPagination, DEFAULT_LIMIT};
use crate::infra::{repository::QueryOptions, Cursor};

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

    fn with_order_by(&mut self, order_by: OrderBy) -> &mut Self {
        self.pagination_mut().order_by = Some(order_by);
        self
    }

    fn with_after(&mut self, after: Option<Cursor>) -> &mut Self {
        self.pagination_mut().after = after;
        self
    }

    fn with_before(&mut self, before: Option<Cursor>) -> &mut Self {
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

    fn apply_conditions(
        query_builder: &mut QueryBuilder<Postgres>,
        conditions: &mut Vec<String>,
        options: &QueryOptions,
        pagination: &QueryPagination,
        cursor_field: &str,
        join_prefix: Option<&str>,
    ) {
        if let Some(timestamp) = &options.timestamp {
            let field = Self::prefix_field("created_at", join_prefix);
            conditions.push(format!(
                "{field} >= to_timestamp({})",
                timestamp.unix_timestamp()
            ));
        }

        if let Some(time_range) = &options.time_range {
            let start_time = time_range.time_since_now();
            let field = Self::prefix_field("created_at", join_prefix);
            conditions.push(format!(
                "{field} >= to_timestamp({})",
                start_time.timestamp()
            ));
        }

        if let Some(from_block) = options.from_block {
            let field = Self::prefix_field("block_height", join_prefix);
            conditions.push(format!("{} >= {}", field, from_block));
        }

        if let Some(ns) = &options.namespace {
            let field = Self::prefix_field("subject", join_prefix);
            conditions.push(format!("{field} LIKE '{ns}%'"));
        }

        if let Some(after) = pagination.after.as_ref() {
            let field = Self::prefix_field(cursor_field, join_prefix);
            conditions.push(format!("{field} > '{after}'"));
        }

        if let Some(before) = pagination.before.as_ref() {
            let field = Self::prefix_field(cursor_field, join_prefix);
            conditions.push(format!("{field} < '{before}'"));
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }
    }

    fn apply_pagination(
        query_builder: &mut QueryBuilder<Postgres>,
        pagination: &QueryPagination,
        cursor_field: &str,
        join_prefix: Option<&str>,
    ) {
        let field = Self::prefix_field(cursor_field, join_prefix);
        match (pagination.first, pagination.last) {
            (Some(first), None) => {
                query_builder.push(format!(" ORDER BY {field} ASC"));
                query_builder.push(format!(" LIMIT {first} "));
                return;
            }
            (None, Some(last)) => {
                query_builder.push(format!(" ORDER BY {field} DESC"));
                query_builder.push(format!(" LIMIT {last} "));
                return;
            }
            _ => {}
        }

        let limit = pagination.limit.unwrap_or(DEFAULT_LIMIT);
        let order_by = pagination.order_by.to_owned().unwrap_or(OrderBy::Desc);
        query_builder.push(format!(" ORDER BY {field} {order_by}"));
        query_builder.push(format!(" LIMIT {limit}"));
        if let Some(offset) = pagination.offset {
            query_builder.push(format!(" OFFSET {offset}"));
        }
    }

    fn prefix_field(field: &str, prefix: Option<&str>) -> String {
        match prefix {
            Some(prefix) => format!("{prefix}{field}"),
            None => field.to_string(),
        }
    }
}

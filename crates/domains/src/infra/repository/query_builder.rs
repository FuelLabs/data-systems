use sqlx::{Postgres, QueryBuilder};

use crate::infra::record::QueryOptions;

pub trait QueryParamsBuilder {
    fn query_builder(&self) -> QueryBuilder<'static, Postgres>;
}

pub trait SubjectQueryBuilder {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres>;
}

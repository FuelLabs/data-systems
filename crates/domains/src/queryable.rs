use actix_web::{dev::Payload, web, Error, FromRequest, HttpRequest};
use async_trait::async_trait;
use fuel_streams_store::{db::DbItem, record::RecordPointer};
use futures::future::{ready, Ready};
use sea_query::{
    Asterisk,
    Condition,
    Expr,
    Iden,
    Order,
    PostgresQueryBuilder,
    Query,
    SelectStatement,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

pub const MAX_FIRST: i32 = 100;
pub const MAX_LAST: i32 = 100;

// NOTE: https://docs.rs/serde_qs/0.14.0/serde_qs/index.html#flatten-workaround
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryPagination {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub after: Option<i32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub before: Option<i32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub first: Option<i32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub last: Option<i32>,
}

impl QueryPagination {
    pub fn with_after(mut self, after: i32) -> Self {
        self.after = Some(after);
        self
    }

    pub fn with_before(mut self, before: i32) -> Self {
        self.before = Some(before);
        self
    }

    pub fn with_first(mut self, first: i32) -> Self {
        self.first = Some(first);
        self
    }

    pub fn with_last(mut self, last: i32) -> Self {
        self.last = Some(last);
        self
    }

    pub fn first(&self) -> Option<i32> {
        self.first
    }

    pub fn last(&self) -> Option<i32> {
        self.last
    }

    pub fn after(&self) -> Option<i32> {
        self.after
    }

    pub fn before(&self) -> Option<i32> {
        self.before
    }
}

impl From<(Option<i32>, Option<i32>, Option<i32>, Option<i32>)>
    for QueryPagination
{
    fn from(val: (Option<i32>, Option<i32>, Option<i32>, Option<i32>)) -> Self {
        Self {
            after: val.0,
            before: val.1,
            first: val.2,
            last: val.3,
        }
    }
}

#[async_trait]
pub trait Queryable: Sized + 'static {
    type Record: DbItem + Into<RecordPointer>;
    type Table: Iden;
    type PaginationColumn: Iden;

    fn table() -> Self::Table;

    fn pagination_column() -> Self::PaginationColumn;

    fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        let pagination = self.pagination();

        // Add after/before conditions
        if let Some(after) = pagination.after() {
            condition =
                condition.add(Expr::col(Self::pagination_column()).gt(after));
        }

        if let Some(before) = pagination.before() {
            condition =
                condition.add(Expr::col(Self::pagination_column()).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Self::table())
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = pagination.first() {
            query = query
                .order_by(Self::pagination_column(), Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = pagination.last() {
            query = query
                .order_by(Self::pagination_column(), Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }

    fn build_condition(&self) -> Condition;

    fn pagination(&self) -> &QueryPagination;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<Self::Record>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, Self::Record>(&sql)
            .fetch_all(executor)
            .await
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct ValidatedQuery<T>(pub T);

impl<T> ValidatedQuery<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for ValidatedQuery<T>
where
    T: serde::de::DeserializeOwned + HasPagination,
{
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let query = web::Query::<T>::from_query(req.query_string());

        match query {
            Ok(q) => {
                // Get pagination and validate
                let pagination = q.pagination();

                match (pagination.first, pagination.last) {
                    (Some(_first), Some(_last)) => {
                        return ready(Err(actix_web::error::ErrorBadRequest(
                            "Cannot specify both 'first' and 'last' pagination parameters"
                        )));
                    }
                    (Some(first), None) => {
                        if first > MAX_FIRST {
                            return ready(Err(
                                actix_web::error::ErrorBadRequest(format!(
                                    "'first' cannot exceed {}",
                                    MAX_FIRST
                                )),
                            ));
                        }
                    }
                    (None, Some(last)) => {
                        if last > MAX_LAST {
                            return ready(Err(
                                actix_web::error::ErrorBadRequest(format!(
                                    "'last' cannot exceed {}",
                                    MAX_LAST
                                )),
                            ));
                        }
                    }
                    _ => {
                        return ready(Err(actix_web::error::ErrorBadRequest(
                            "Either 'first' or 'last' pagination parameter must be specified"
                        )));
                    }
                }

                ready(Ok(ValidatedQuery(q.into_inner())))
            }
            Err(e) => ready(Err(actix_web::error::ErrorBadRequest(e))),
        }
    }
}

/// A trait to extract pagination from queries
pub trait HasPagination {
    fn pagination(&self) -> &QueryPagination;
}

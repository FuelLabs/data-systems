use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Query as AxumQuery},
    http::{request::Parts, StatusCode},
};
use fuel_streams_store::{db::DbItem, record::RecordPointer};
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
use utoipa::ToSchema;

pub const MAX_FIRST: i32 = 100;
pub const MAX_LAST: i32 = 100;

#[serde_as]
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq, ToSchema,
)]
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

        if let Some(after) = pagination.after() {
            condition =
                condition.add(Expr::col(Self::pagination_column()).gt(after));
        }

        if let Some(before) = pagination.before() {
            condition =
                condition.add(Expr::col(Self::pagination_column()).lt(before));
        }

        let mut query = Query::select();
        query
            .column(Asterisk)
            .from(Self::table())
            .cond_where(condition);

        if let Some(first) = pagination.first() {
            query
                .order_by(Self::pagination_column(), Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = pagination.last() {
            query
                .order_by(Self::pagination_column(), Order::Desc)
                .limit(last as u64);
        }

        query
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
    ) -> Result<Vec<Self::Record>, QueryableError>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, Self::Record>(&sql)
            .fetch_all(executor)
            .await
            .map_err(QueryableError::from)
    }
}

#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    utoipa::ToSchema,
)]
pub struct ValidatedQuery<T>(pub T);

impl<T> ValidatedQuery<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryableError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Cannot specify both 'first' and 'last' pagination parameters")]
    BothFirstAndLastSpecified,

    #[error("'first' cannot exceed {0}")]
    FirstExceedsMaximum(i32),

    #[error("'last' cannot exceed {0}")]
    LastExceedsMaximum(i32),

    #[error("Either 'first' or 'last' pagination parameter must be specified")]
    NeitherFirstNorLastSpecified,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl axum::response::IntoResponse for QueryableError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            QueryableError::BadRequest(_) => StatusCode::BAD_REQUEST,
            QueryableError::BothFirstAndLastSpecified => {
                StatusCode::BAD_REQUEST
            }
            QueryableError::FirstExceedsMaximum(_) => StatusCode::BAD_REQUEST,
            QueryableError::LastExceedsMaximum(_) => StatusCode::BAD_REQUEST,
            QueryableError::NeitherFirstNorLastSpecified => {
                StatusCode::BAD_REQUEST
            }
            QueryableError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = self.to_string();
        (status, body).into_response()
    }
}

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned + HasPagination + Send + 'static,
{
    type Rejection = QueryableError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = AxumQuery::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| QueryableError::BadRequest(e.to_string()))?;

        let q = query.0;
        let pagination = q.pagination();

        match (pagination.first, pagination.last) {
            (Some(_first), Some(_last)) => {
                return Err(QueryableError::BothFirstAndLastSpecified);
            }
            (Some(first), None) => {
                if first > MAX_FIRST {
                    return Err(QueryableError::FirstExceedsMaximum(MAX_FIRST));
                }
            }
            (None, Some(last)) => {
                if last > MAX_LAST {
                    return Err(QueryableError::LastExceedsMaximum(MAX_LAST));
                }
            }
            _ => {
                return Err(QueryableError::NeitherFirstNorLastSpecified);
            }
        }

        Ok(ValidatedQuery(q))
    }
}

pub trait HasPagination {
    fn pagination(&self) -> &QueryPagination;
}

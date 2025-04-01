use axum::{
    extract::{FromRequestParts, Query as AxumQuery},
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use sqlx::{Postgres, QueryBuilder};
use utoipa::ToSchema;

pub const MAX_LIMIT: i32 = 1000;
pub const DEFAULT_LIMIT: i32 = 100;

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
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub limit: Option<i32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub offset: Option<i32>,
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

    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
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

    pub fn limit(&self) -> Option<i32> {
        self.limit
    }

    pub fn offset(&self) -> Option<i32> {
        self.offset
    }

    pub fn increment_offset(&mut self) {
        if let Some(offset) = self.offset {
            self.offset = Some(offset + self.limit.unwrap_or(DEFAULT_LIMIT));
        }
    }

    pub fn apply_on_query(
        &self,
        query_builder: &mut QueryBuilder<Postgres>,
        cursor_field: &str,
    ) {
        let mut conditions = Vec::new();

        if let Some(after) = self.after {
            conditions.push(format!("{} > ", cursor_field));
            query_builder.push_bind(after);
            query_builder.push(" ");
        }

        if let Some(before) = self.before {
            conditions.push(format!("{} < ", cursor_field));
            query_builder.push_bind(before);
            query_builder.push(" ");
        }

        if !conditions.is_empty() {
            let existing_where = query_builder.sql().contains("WHERE");
            if !existing_where {
                query_builder.push(" WHERE ");
            } else {
                query_builder.push(" AND ");
            }
            query_builder.push(conditions.join(" AND "));
        }

        // Determine limit and offset from QueryPagination
        let limit = self
            .limit
            .or(self.first)
            .unwrap_or(DEFAULT_LIMIT)
            .min(MAX_LIMIT);

        let offset = self.offset.filter(|&o| o >= 0);

        // Apply ordering and pagination
        match (self.first.is_some(), self.last.is_some()) {
            (true, false) => {
                query_builder.push(format!(" ORDER BY {} ASC", cursor_field));
                query_builder.push(" LIMIT ");
                query_builder.push_bind(limit);
                if let Some(offset) = offset {
                    query_builder.push(" OFFSET ");
                    query_builder.push_bind(offset);
                }
            }
            (false, true) => {
                query_builder.push(format!(" ORDER BY {} DESC", cursor_field));
                query_builder.push(" LIMIT ");
                query_builder.push_bind(limit);
                if let Some(offset) = offset {
                    query_builder.push(" OFFSET ");
                    query_builder.push_bind(offset);
                }
            }
            _ => {
                query_builder.push(format!(" ORDER BY {} ASC", cursor_field));
                query_builder.push(" LIMIT ");
                query_builder.push_bind(limit);
                if let Some(offset) = offset {
                    query_builder.push(" OFFSET ");
                    query_builder.push_bind(offset);
                }
            }
        }
    }
}

impl
    From<(
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
    )> for QueryPagination
{
    fn from(
        val: (
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
        ),
    ) -> Self {
        Self {
            after: val.0,
            before: val.1,
            first: val.2,
            last: val.3,
            limit: val.4,
            offset: val.5,
        }
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
pub enum ValidatedQueryError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(
        "Cannot mix cursor-based (first/last) and limit/offset pagination"
    )]
    MixedPaginationTypes,
    #[error("'first' cannot exceed {0}")]
    FirstExceedsMaximum(i32),
    #[error("'last' cannot exceed {0}")]
    LastExceedsMaximum(i32),
    #[error("'limit' cannot exceed {0}")]
    LimitExceedsMaximum(i32),
    #[error("'offset' cannot be negative")]
    NegativeOffset,
}

impl axum::response::IntoResponse for ValidatedQueryError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::BAD_REQUEST;
        let body = self.to_string();
        (status, body).into_response()
    }
}

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned + HasPagination + Send + 'static,
{
    type Rejection = ValidatedQueryError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = AxumQuery::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| ValidatedQueryError::BadRequest(e.to_string()))?;

        let q = query.0;
        let pagination = q.pagination();

        match (
            pagination.first,
            pagination.last,
            pagination.limit,
            pagination.offset,
        ) {
            (Some(first), None, None, None) => {
                if first > MAX_LIMIT {
                    return Err(ValidatedQueryError::FirstExceedsMaximum(
                        MAX_LIMIT,
                    ));
                }
            }
            (None, Some(last), None, None) => {
                if last > MAX_LIMIT {
                    return Err(ValidatedQueryError::LastExceedsMaximum(
                        MAX_LIMIT,
                    ));
                }
            }
            (None, None, Some(limit), offset) => {
                if limit > MAX_LIMIT {
                    return Err(ValidatedQueryError::LimitExceedsMaximum(
                        MAX_LIMIT,
                    ));
                }
                if let Some(offset) = offset {
                    if offset < 0 {
                        return Err(ValidatedQueryError::NegativeOffset);
                    }
                }
            }
            (None, None, None, None) => {}
            _ => {
                return Err(ValidatedQueryError::MixedPaginationTypes);
            }
        }

        Ok(ValidatedQuery(q))
    }
}

pub trait HasPagination {
    fn pagination(&self) -> &QueryPagination;
}

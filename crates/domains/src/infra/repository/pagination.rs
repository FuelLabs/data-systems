use std::fmt::Display;

use axum::{
    extract::{FromRequestParts, Query as AxumQuery},
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use utoipa::ToSchema;

use crate::infra::Cursor;

pub const MAX_LIMIT: i32 = 1000;
pub const DEFAULT_LIMIT: i32 = 100;

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq, ToSchema,
)]
pub enum OrderBy {
    #[default]
    Desc,
    Asc,
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderBy::Desc => write!(f, "DESC"),
            OrderBy::Asc => write!(f, "ASC"),
        }
    }
}

#[serde_as]
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq, ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct QueryPagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Cursor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Cursor>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub first: Option<i32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub last: Option<i32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub limit: Option<i32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub offset: Option<i32>,
    pub order_by: Option<OrderBy>,
}

impl QueryPagination {
    pub fn with_after(mut self, after: Cursor) -> Self {
        self.after = Some(after);
        self
    }

    pub fn with_before(mut self, before: Cursor) -> Self {
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

    pub fn with_order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by = Some(order_by);
        self
    }

    pub fn first(&self) -> Option<i32> {
        self.first
    }

    pub fn last(&self) -> Option<i32> {
        self.last
    }

    pub fn after(&self) -> Option<&Cursor> {
        self.after.as_ref()
    }

    pub fn before(&self) -> Option<&Cursor> {
        self.before.as_ref()
    }

    pub fn limit(&self) -> Option<i32> {
        self.limit
    }

    pub fn offset(&self) -> Option<i32> {
        self.offset
    }

    pub fn order_by(&self) -> Option<&OrderBy> {
        self.order_by.as_ref()
    }

    pub fn increment_offset(&mut self) {
        if let Some(offset) = self.offset {
            self.offset = Some(offset + self.limit.unwrap_or(DEFAULT_LIMIT));
        }
    }
}

impl
    From<(
        Option<Cursor>,
        Option<Cursor>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<OrderBy>,
    )> for QueryPagination
{
    fn from(
        val: (
            Option<Cursor>,
            Option<Cursor>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<OrderBy>,
        ),
    ) -> Self {
        Self {
            after: val.0,
            before: val.1,
            first: val.2,
            last: val.3,
            limit: val.4,
            offset: val.5,
            order_by: val.6,
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
    #[error("Invalid pagination: Cannot mix cursor-based pagination (after/before) with offset-based pagination (limit/offset)")]
    MixedPaginationStrategy,
    #[error("Invalid pagination: Cannot use both 'after' and 'before' cursors simultaneously")]
    ConflictingCursors,
    #[error("Invalid pagination: Cannot use both 'first' and 'last' parameters simultaneously")]
    ConflictingFirstLast,
    #[error("Invalid pagination: 'first' parameter must be between 1 and {0}")]
    InvalidFirst(i32),
    #[error("Invalid pagination: 'last' parameter must be between 1 and {0}")]
    InvalidLast(i32),
    #[error("Invalid pagination: 'limit' parameter must be between 1 and {0}")]
    InvalidLimit(i32),
    #[error("Invalid pagination: 'offset' parameter cannot be negative")]
    NegativeOffset,
    #[error("Invalid pagination: 'first' parameter is required when using 'after' cursor")]
    MissingFirstWithAfter,
    #[error("Invalid pagination: 'last' parameter is required when using 'before' cursor")]
    MissingLastWithBefore,
    #[error("Invalid pagination: 'order_by' cannot be used with cursor-based pagination")]
    OrderByWithCursor,
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

        // Check for mixing cursor-based and offset-based pagination
        if (pagination.after.is_some() || pagination.before.is_some())
            && (pagination.limit.is_some() || pagination.offset.is_some())
        {
            return Err(ValidatedQueryError::MixedPaginationStrategy);
        }

        // Check for conflicting cursors
        if pagination.after.is_some() && pagination.before.is_some() {
            return Err(ValidatedQueryError::ConflictingCursors);
        }

        // Check for conflicting first/last
        if pagination.first.is_some() && pagination.last.is_some() {
            return Err(ValidatedQueryError::ConflictingFirstLast);
        }

        // Check that order_by is not used with cursor pagination
        if (pagination.after.is_some() || pagination.before.is_some())
            && pagination.order_by.is_some()
        {
            return Err(ValidatedQueryError::OrderByWithCursor);
        }

        // Validate cursor-based pagination
        match (
            pagination.after.clone(),
            pagination.before.clone(),
            pagination.first,
            pagination.last,
        ) {
            (Some(_), None, None, _) => {
                return Err(ValidatedQueryError::MissingFirstWithAfter);
            }
            (None, Some(_), _, None) => {
                return Err(ValidatedQueryError::MissingLastWithBefore);
            }
            (Some(_), None, Some(first), None) => {
                if first <= 0 || first > MAX_LIMIT {
                    return Err(ValidatedQueryError::InvalidFirst(MAX_LIMIT));
                }
            }
            (None, Some(_), None, Some(last)) => {
                if last <= 0 || last > MAX_LIMIT {
                    return Err(ValidatedQueryError::InvalidLast(MAX_LIMIT));
                }
            }
            (None, None, Some(first), None) => {
                if first <= 0 || first > MAX_LIMIT {
                    return Err(ValidatedQueryError::InvalidFirst(MAX_LIMIT));
                }
            }
            (None, None, None, Some(last)) => {
                if last <= 0 || last > MAX_LIMIT {
                    return Err(ValidatedQueryError::InvalidLast(MAX_LIMIT));
                }
            }
            (None, None, None, None) => {
                // Validate offset-based pagination
                if let Some(limit) = pagination.limit {
                    if limit <= 0 || limit > MAX_LIMIT {
                        return Err(ValidatedQueryError::InvalidLimit(
                            MAX_LIMIT,
                        ));
                    }
                }
                if let Some(offset) = pagination.offset {
                    if offset < 0 {
                        return Err(ValidatedQueryError::NegativeOffset);
                    }
                }
            }
            _ => return Err(ValidatedQueryError::MixedPaginationStrategy),
        }

        Ok(ValidatedQuery(q))
    }
}

pub trait HasPagination {
    fn pagination(&self) -> &QueryPagination;
}

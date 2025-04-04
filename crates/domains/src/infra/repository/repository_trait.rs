use std::fmt;

use async_trait::async_trait;
use sqlx::{Acquire, PgConnection, PgExecutor, Postgres};

use super::{QueryParamsBuilder, RepositoryResult};
use crate::infra::{db::DbItem, record::RecordPointer, DbTransaction};

pub type DbConnection = PgConnection;

#[async_trait]
pub trait Repository: Clone + Sized + Send + Sync + 'static {
    type Item: DbItem + Into<RecordPointer>;
    type QueryParams: fmt::Debug
        + QueryParamsBuilder
        + Send
        + Sync
        + 'static
        + Clone;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>;

    async fn insert_with_transaction(
        tx: &mut DbTransaction,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item> {
        Self::insert(&mut **tx, db_item).await
    }

    async fn find_one<'e, 'c: 'e, E>(
        executor: E,
        params: &Self::QueryParams,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let mut query = params.query_builder();
        let result = query
            .build_query_as::<Self::Item>()
            .fetch_one(executor)
            .await?;
        Ok(result)
    }

    async fn find_one_with_db_tx(
        tx: &mut DbTransaction,
        params: &Self::QueryParams,
    ) -> RepositoryResult<Self::Item> {
        Self::find_one(&mut **tx, params).await
    }

    async fn find_many<'e, 'c: 'e, E>(
        executor: E,
        params: &Self::QueryParams,
    ) -> RepositoryResult<Vec<Self::Item>>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let mut query = params.query_builder();
        let result = query
            .build_query_as::<Self::Item>()
            .fetch_all(executor)
            .await?;
        Ok(result)
    }

    async fn find_many_with_db_tx(
        tx: &mut DbTransaction,
        params: &Self::QueryParams,
    ) -> RepositoryResult<Vec<Self::Item>> {
        Self::find_many(&mut **tx, params).await
    }
}

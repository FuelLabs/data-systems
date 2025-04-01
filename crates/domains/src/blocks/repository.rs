use async_trait::async_trait;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Block, BlockDbItem, BlocksQuery};
use crate::infra::{
    db::Db,
    repository::{Repository, RepositoryError, RepositoryResult},
    QueryOptions,
};

#[async_trait]
impl Repository for Block {
    type Item = BlockDbItem;
    type QueryParams = BlocksQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, BlockDbItem>(
            "WITH upsert AS (
                INSERT INTO blocks (subject, producer_address, block_da_height, block_height, value, created_at, published_at, block_propagation_ms)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (subject) DO UPDATE SET
                    producer_address = EXCLUDED.producer_address,
                    block_da_height = EXCLUDED.block_da_height,
                    block_height = EXCLUDED.block_height,
                    value = EXCLUDED.value,
                    created_at = EXCLUDED.created_at,
                    published_at = $7,
                    block_propagation_ms = $8
                RETURNING *
            )
            SELECT * FROM upsert"
        )
        .bind(db_item.subject.to_owned())
        .bind(db_item.producer_address.to_owned())
        .bind(db_item.block_da_height)
        .bind(db_item.block_height)
        .bind(db_item.value.to_owned())
        .bind(db_item.created_at)
        .bind(published_at)
        .bind(db_item.block_propagation_ms)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

impl Block {
    pub async fn find_last_block_height(
        db: &Db,
        options: &QueryOptions,
    ) -> RepositoryResult<BlockHeight> {
        let select = "SELECT block_height FROM blocks".to_string();
        let mut query_builder = sqlx::QueryBuilder::new(select);
        if let Some(ns) = options.namespace.as_ref() {
            query_builder
                .push(" WHERE subject LIKE ")
                .push_bind(format!("{}%", ns));
        }
        query_builder.push(" ORDER BY block_height DESC LIMIT 1");
        let query = query_builder.build_query_as::<(i64,)>();
        let record: Option<(i64,)> = query.fetch_optional(&db.pool).await?;
        Ok(record.map(|(height,)| height.into()).unwrap_or_default())
    }
}

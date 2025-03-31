use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Predicate, PredicateDbItem, PredicatesQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Predicate {
    type Item = PredicateDbItem;
    type QueryParams = PredicatesQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let mut tx = sqlx::Acquire::begin(executor)
            .await
            .map_err(RepositoryError::Insert)?;

        let predicate_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO predicates (
                value,
                blob_id,
                predicate_address,
                created_at,
                published_at
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (predicate_address) DO UPDATE
            SET blob_id = EXCLUDED.blob_id,
                value = EXCLUDED.value,
                published_at = EXCLUDED.published_at
            RETURNING id",
        )
        .bind(db_item.value.to_owned())
        .bind(db_item.blob_id.to_owned())
        .bind(db_item.predicate_address.to_owned())
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(RepositoryError::Insert)?;

        // Insert the transaction relationship
        sqlx::query(
            "INSERT INTO predicate_transactions (
                predicate_id,
                subject,
                block_height,
                tx_id,
                tx_index,
                input_index
            )
            VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(predicate_id)
        .bind(db_item.subject.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::Insert)?;

        tx.commit().await.map_err(RepositoryError::Insert)?;

        Ok(PredicateDbItem {
            published_at,
            ..db_item.to_owned()
        })
    }
}

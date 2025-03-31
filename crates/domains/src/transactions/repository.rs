use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Transaction, TransactionDbItem, TransactionsQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Transaction {
    type Item = TransactionDbItem;
    type QueryParams = TransactionsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, TransactionDbItem>(
            "WITH upsert AS (
                INSERT INTO transactions (
                    subject, value, block_height, tx_id, tx_index,
                    tx_status, type, blob_id, created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (subject) DO UPDATE SET
                    tx_id = EXCLUDED.tx_id,
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_index = EXCLUDED.tx_index,
                    tx_status = EXCLUDED.tx_status,
                    type = EXCLUDED.type,
                    blob_id = EXCLUDED.blob_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $10
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(db_item.subject.clone())
        .bind(db_item.value.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.tx_status.to_owned())
        .bind(db_item.r#type.to_owned())
        .bind(db_item.blob_id.to_owned())
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

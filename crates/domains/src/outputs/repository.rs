use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Output, OutputDbItem, OutputsQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Output {
    type Item = OutputDbItem;
    type QueryParams = OutputsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, OutputDbItem>(
            "WITH upsert AS (
                INSERT INTO outputs (
                    subject, value, block_height, tx_id, tx_index,
                    output_index, output_type, to_address, asset_id, contract_id,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    output_index = EXCLUDED.output_index,
                    output_type = EXCLUDED.output_type,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $12
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(db_item.subject.clone())
        .bind(db_item.value.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.output_index)
        .bind(db_item.output_type.to_owned())
        .bind(db_item.to_address.to_owned())
        .bind(db_item.asset_id.to_owned())
        .bind(db_item.contract_id.to_owned())
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

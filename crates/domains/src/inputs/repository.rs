use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Input, InputDbItem, InputsQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Input {
    type Item = InputDbItem;
    type QueryParams = InputsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, InputDbItem>(
            "WITH upsert AS (
                INSERT INTO inputs (
                    subject, value, block_height, tx_id, tx_index,
                    input_index, input_type, owner_id, asset_id,
                    contract_id, sender_address, recipient_address,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    input_index = EXCLUDED.input_index,
                    input_type = EXCLUDED.input_type,
                    owner_id = EXCLUDED.owner_id,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    sender_address = EXCLUDED.sender_address,
                    recipient_address = EXCLUDED.recipient_address,
                    created_at = EXCLUDED.created_at,
                    published_at = $14
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(db_item.subject.clone())
        .bind(db_item.value.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(db_item.input_type.to_owned())
        .bind(db_item.owner_id.to_owned())
        .bind(db_item.asset_id.to_owned())
        .bind(db_item.contract_id.to_owned())
        .bind(db_item.sender_address.to_owned())
        .bind(db_item.recipient_address.to_owned())
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

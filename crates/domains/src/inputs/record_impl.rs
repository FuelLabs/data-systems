use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity},
};
use fuel_streams_types::BlockTimestamp;
use sqlx::PgExecutor;

use super::{Input, InputDbItem};

impl DataEncoder for Input {
    type Err = DbError;
}

#[async_trait]
impl Record for Input {
    type DbItem = InputDbItem;

    const ENTITY: RecordEntity = RecordEntity::Input;
    const ORDER_PROPS: &'static [&'static str] = &["tx_index", "input_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: Self::DbItem,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
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
        .bind(db_item.subject)
        .bind(db_item.value)
        .bind(db_item.block_height)
        .bind(db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(db_item.input_type)
        .bind(db_item.owner_id)
        .bind(db_item.asset_id)
        .bind(db_item.contract_id)
        .bind(db_item.sender_address)
        .bind(db_item.recipient_address)
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

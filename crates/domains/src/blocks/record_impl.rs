use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity},
};
use fuel_streams_types::BlockTimestamp;
use sqlx::PgExecutor;

use super::{Block, BlockDbItem};

impl DataEncoder for Block {
    type Err = DbError;
}

#[async_trait]
impl Record for Block {
    type DbItem = BlockDbItem;

    const ENTITY: RecordEntity = RecordEntity::Block;
    const ORDER_PROPS: &'static [&'static str] = &["block_height"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: Self::DbItem,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
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
        .bind(db_item.subject)
        .bind(db_item.producer_address)
        .bind(db_item.block_da_height)
        .bind(db_item.block_height)
        .bind(db_item.value)
        .bind(db_item.created_at)
        .bind(published_at)
        .bind(db_item.block_propagation_ms)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

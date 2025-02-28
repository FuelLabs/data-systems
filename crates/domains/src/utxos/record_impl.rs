use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity},
};
use fuel_streams_types::BlockTimestamp;
use sqlx::PgExecutor;

use super::{Utxo, UtxoDbItem};

impl DataEncoder for Utxo {
    type Err = DbError;
}

#[async_trait]
impl Record for Utxo {
    type DbItem = UtxoDbItem;

    const ENTITY: RecordEntity = RecordEntity::Utxo;
    const ORDER_PROPS: &'static [&'static str] = &["tx_index", "input_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: Self::DbItem,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let published_at: BlockTimestamp = chrono::Utc::now().into();
        let record = sqlx::query_as::<_, UtxoDbItem>(
            "WITH upsert AS (
                INSERT INTO utxos (
                    subject, value, block_height, tx_id, tx_index,
                    input_index, utxo_type, utxo_id, created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    input_index = EXCLUDED.input_index,
                    utxo_type = EXCLUDED.utxo_type,
                    utxo_id = EXCLUDED.utxo_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $10
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
        .bind(db_item.utxo_type)
        .bind(db_item.utxo_id)
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

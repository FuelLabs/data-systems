use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};
use sqlx::PgExecutor;

use super::{Output, OutputDbItem};

impl DataEncoder for Output {
    type Err = DbError;
}

#[async_trait]
impl Record for Output {
    type DbItem = OutputDbItem;

    const ENTITY: RecordEntity = RecordEntity::Output;
    const ORDER_PROPS: &'static [&'static str] = &["tx_index", "output_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let db_item = OutputDbItem::try_from(packet)?;
        let record = sqlx::query_as::<_, OutputDbItem>(
            "WITH upsert AS (
                INSERT INTO outputs (
                    subject, value, block_height, tx_id, tx_index,
                    output_index, output_type, to_address, asset_id, contract_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    output_index = EXCLUDED.output_index,
                    output_type = EXCLUDED.output_type,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(db_item.subject)
        .bind(db_item.value)
        .bind(db_item.block_height)
        .bind(db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.output_index)
        .bind(db_item.output_type)
        .bind(db_item.to_address)
        .bind(db_item.asset_id)
        .bind(db_item.contract_id)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

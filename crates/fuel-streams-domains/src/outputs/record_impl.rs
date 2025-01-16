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
    const ORDER_PROPS: &'static [&'static str] =
        &["block_height", "tx_index", "output_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let db_item = OutputDbItem::try_from(packet)?;
        let record = sqlx::query_as!(
            Self::DbItem,
            r#"
                INSERT INTO outputs (
                    subject, value, block_height, tx_id, tx_index,
                    output_index, output_type, to_address, asset_id, contract_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING subject, value, block_height, tx_id, tx_index,
                    output_index, output_type, to_address, asset_id, contract_id
                "#,
            db_item.subject,
            db_item.value,
            db_item.block_height,
            db_item.tx_id,
            db_item.tx_index,
            db_item.output_index,
            db_item.output_type,
            db_item.to_address,
            db_item.asset_id,
            db_item.contract_id
        )
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

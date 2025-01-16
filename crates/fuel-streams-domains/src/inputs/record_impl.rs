use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};
use sqlx::PgExecutor;

use super::{Input, InputDbItem};

impl DataEncoder for Input {
    type Err = DbError;
}

#[async_trait]
impl Record for Input {
    type DbItem = InputDbItem;

    const ENTITY: RecordEntity = RecordEntity::Input;
    const ORDER_PROPS: &'static [&'static str] =
        &["block_height", "tx_index", "input_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let db_item = InputDbItem::try_from(packet)?;
        let record = sqlx::query_as!(
            Self::DbItem,
            r#"
                INSERT INTO inputs (
                    subject, value, block_height, tx_id, tx_index,
                    input_index, input_type, owner_id, asset_id,
                    contract_id, sender_address, recipient_address
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING subject, value, block_height, tx_id, tx_index,
                    input_index, input_type, owner_id, asset_id,
                    contract_id, sender_address, recipient_address
                "#,
            db_item.subject,
            db_item.value,
            db_item.block_height,
            db_item.tx_id,
            db_item.tx_index,
            db_item.input_index,
            db_item.input_type,
            db_item.owner_id,
            db_item.asset_id,
            db_item.contract_id,
            db_item.sender_address,
            db_item.recipient_address
        )
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

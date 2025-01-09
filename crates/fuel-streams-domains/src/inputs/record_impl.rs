use async_trait::async_trait;
use fuel_streams_store::{
    db::{Db, DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};

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

    async fn insert(
        &self,
        db: &Db,
        packet: &RecordPacket<Self>,
    ) -> DbResult<Self::DbItem> {
        let db_item = InputDbItem::try_from(packet)?;
        let record = sqlx::query_as::<_, Self::DbItem>(
            r#"
            INSERT INTO inputs (
                subject, value, block_height, tx_id, tx_index,
                input_index, input_type, owner_id, asset_id,
                contract_id, sender, recipient
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING subject, value, block_height, tx_id, tx_index,
                input_index, input_type, owner_id, asset_id,
                contract_id, sender, recipient
            "#,
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
        .bind(db_item.sender)
        .bind(db_item.recipient)
        .fetch_one(&db.pool)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

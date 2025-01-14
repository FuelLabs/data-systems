use async_trait::async_trait;
use fuel_streams_store::{
    db::{Db, DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};

use super::{Receipt, ReceiptDbItem};

impl DataEncoder for Receipt {
    type Err = DbError;
}

#[async_trait]
impl Record for Receipt {
    type DbItem = ReceiptDbItem;

    const ENTITY: RecordEntity = RecordEntity::Receipt;
    const ORDER_PROPS: &'static [&'static str] =
        &["block_height", "tx_index", "receipt_index"];

    async fn insert(
        &self,
        db: &Db,
        packet: &RecordPacket<Self>,
    ) -> DbResult<Self::DbItem> {
        let db_item = ReceiptDbItem::try_from(packet)?;
        let record = sqlx::query_as!(
            Self::DbItem,
            r#"
            INSERT INTO receipts (
                subject, value, block_height, tx_id, tx_index, receipt_index,
                receipt_type, from_contract_id, to_contract_id, to_address,
                asset_id, contract_id, sub_id, sender_address, recipient_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING subject, value, block_height, tx_id, tx_index, receipt_index,
                receipt_type, from_contract_id, to_contract_id, to_address,
                asset_id, contract_id, sub_id, sender_address, recipient_address
            "#,
            db_item.subject,
            db_item.value,
            db_item.block_height,
            db_item.tx_id,
            db_item.tx_index,
            db_item.receipt_index,
            db_item.receipt_type,
            db_item.from_contract_id,
            db_item.to_contract_id,
            db_item.to_address,
            db_item.asset_id,
            db_item.contract_id,
            db_item.sub_id,
            db_item.sender_address,
            db_item.recipient_address
        )
        .fetch_one(&db.pool)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

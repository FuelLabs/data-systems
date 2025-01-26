use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};
use sqlx::PgExecutor;

use super::{Receipt, ReceiptDbItem};

impl DataEncoder for Receipt {
    type Err = DbError;
}

#[async_trait]
impl Record for Receipt {
    type DbItem = ReceiptDbItem;

    const ENTITY: RecordEntity = RecordEntity::Receipt;
    const ORDER_PROPS: &'static [&'static str] = &["tx_index", "receipt_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let db_item = ReceiptDbItem::try_from(packet)?;
        let record = sqlx::query_as::<_, ReceiptDbItem>(
            "WITH upsert AS (
                INSERT INTO receipts (
                    subject, value, block_height, tx_id, tx_index, receipt_index,
                    receipt_type, from_contract_id, to_contract_id, to_address,
                    asset_id, contract_id, sub_id, sender_address, recipient_address
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    receipt_index = EXCLUDED.receipt_index,
                    receipt_type = EXCLUDED.receipt_type,
                    from_contract_id = EXCLUDED.from_contract_id,
                    to_contract_id = EXCLUDED.to_contract_id,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    sub_id = EXCLUDED.sub_id,
                    sender_address = EXCLUDED.sender_address,
                    recipient_address = EXCLUDED.recipient_address
                RETURNING *
            )
            SELECT * FROM upsert"
        )
        .bind(db_item.subject)
        .bind(db_item.value)
        .bind(db_item.block_height)
        .bind(db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.receipt_index)
        .bind(db_item.receipt_type)
        .bind(db_item.from_contract_id)
        .bind(db_item.to_contract_id)
        .bind(db_item.to_address)
        .bind(db_item.asset_id)
        .bind(db_item.contract_id)
        .bind(db_item.sub_id)
        .bind(db_item.sender_address)
        .bind(db_item.recipient_address)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

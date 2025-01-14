use async_trait::async_trait;
use fuel_streams_store::{
    db::{Db, DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};

use super::{Transaction, TransactionDbItem};

impl DataEncoder for Transaction {
    type Err = DbError;
}

#[async_trait]
impl Record for Transaction {
    type DbItem = TransactionDbItem;

    const ENTITY: RecordEntity = RecordEntity::Transaction;
    const ORDER_PROPS: &'static [&'static str] = &["block_height", "tx_index"];

    async fn insert(
        &self,
        db: &Db,
        packet: &RecordPacket<Self>,
    ) -> DbResult<Self::DbItem> {
        let db_item = TransactionDbItem::try_from(packet)?;
        let record = sqlx::query_as::<_, Self::DbItem>(
            r#"
            INSERT INTO transactions (
                subject, value, block_height, tx_id, tx_index, tx_status, kind
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING subject, value, block_height, tx_id, tx_index, tx_status, kind
            "#,
        )
        .bind(db_item.subject)
        .bind(db_item.value)
        .bind(db_item.block_height)
        .bind(db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.tx_status)
        .bind(db_item.kind)
        .fetch_one(&db.pool)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

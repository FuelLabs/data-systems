use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};
use sqlx::PgExecutor;

use super::{Transaction, TransactionDbItem};

impl DataEncoder for Transaction {
    type Err = DbError;
}

#[async_trait]
impl Record for Transaction {
    type DbItem = TransactionDbItem;

    const ENTITY: RecordEntity = RecordEntity::Transaction;
    const ORDER_PROPS: &'static [&'static str] = &["block_height", "tx_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let db_item = TransactionDbItem::try_from(packet)?;
        let record = sqlx::query_as::<_, TransactionDbItem>(
            "INSERT INTO transactions (
                subject, value, block_height, tx_id, tx_index, tx_status, kind
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING subject, value, block_height, tx_id, tx_index, tx_status, kind"
        )
        .bind(db_item.subject)
        .bind(db_item.value)
        .bind(db_item.block_height)
        .bind(db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.tx_status)
        .bind(db_item.kind)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

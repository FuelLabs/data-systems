use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};
use sqlx::PgExecutor;

use super::{Utxo, UtxoDbItem, UtxoStoreItem};

impl DataEncoder for Utxo {
    type Err = DbError;
}

#[async_trait]
impl Record for Utxo {
    type DbItem = UtxoDbItem;
    type StoreItem = UtxoStoreItem;

    const ENTITY: RecordEntity = RecordEntity::Utxo;
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
        let db_item = UtxoDbItem::try_from(packet)?;
        let record = sqlx::query_as::<_, UtxoDbItem>(
            "INSERT INTO utxos (
                subject, value, block_height, tx_id, tx_index,
                input_index, utxo_type, utxo_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING subject, value, block_height, tx_id, tx_index,
                input_index, utxo_type, utxo_id",
        )
        .bind(db_item.subject)
        .bind(db_item.value)
        .bind(db_item.block_height)
        .bind(db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(db_item.utxo_type)
        .bind(db_item.utxo_id)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

use async_trait::async_trait;
use fuel_streams_store::{
    db::{Db, DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};

use super::{Utxo, UtxoDbItem};

impl DataEncoder for Utxo {
    type Err = DbError;
}

#[async_trait]
impl Record for Utxo {
    type DbItem = UtxoDbItem;

    const ENTITY: RecordEntity = RecordEntity::Utxo;
    const ORDER_PROPS: &'static [&'static str] =
        &["block_height", "tx_index", "input_index"];

    async fn insert(
        &self,
        db: &Db,
        packet: &RecordPacket<Self>,
    ) -> DbResult<Self::DbItem> {
        let db_item = UtxoDbItem::try_from(packet)?;
        let record = sqlx::query_as!(
            Self::DbItem,
            r#"
            INSERT INTO utxos (
                subject, value, block_height, tx_id, tx_index,
                input_index, utxo_type, utxo_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING subject, value, block_height, tx_id, tx_index,
                input_index, utxo_type, utxo_id
            "#,
            db_item.subject,
            db_item.value,
            db_item.block_height,
            db_item.tx_id,
            db_item.tx_index,
            db_item.input_index,
            db_item.utxo_type,
            db_item.utxo_id
        )
        .fetch_one(&db.pool)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

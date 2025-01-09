use async_trait::async_trait;
use fuel_streams_store::{
    db::{Db, DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};

use super::{Block, BlockDbItem};

impl DataEncoder for Block {
    type Err = DbError;
}

#[async_trait]
impl Record for Block {
    type DbItem = BlockDbItem;

    const ENTITY: RecordEntity = RecordEntity::Block;
    const ORDER_PROPS: &'static [&'static str] = &["height"];

    async fn insert(
        &self,
        db: &Db,
        packet: &RecordPacket<Self>,
    ) -> DbResult<Self::DbItem> {
        let db_item = BlockDbItem::try_from(packet)?;
        let record = sqlx::query_as::<_, Self::DbItem>(
            r#"
            INSERT INTO blocks (subject, producer_address, height, value)
            VALUES ($1, $2, $3, $4)
            RETURNING subject, producer_address, height, value
            "#,
        )
        .bind(db_item.subject)
        .bind(db_item.producer_address)
        .bind(db_item.height)
        .bind(db_item.value)
        .fetch_one(&db.pool)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

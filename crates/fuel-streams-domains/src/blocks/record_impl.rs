use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity, RecordPacket},
};
use sqlx::PgExecutor;

use super::{Block, BlockDbItem};

impl DataEncoder for Block {
    type Err = DbError;
}

#[async_trait]
impl Record for Block {
    type DbItem = BlockDbItem;
    const ENTITY: RecordEntity = RecordEntity::Block;
    const ORDER_PROPS: &'static [&'static str] = &["block_height"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let db_item = BlockDbItem::try_from(packet)?;
        let record = sqlx::query_as!(
            BlockDbItem,
            r#"
            INSERT INTO blocks (subject, producer_address, block_height, value)
            VALUES ($1, $2, $3, $4)
            RETURNING subject, producer_address, block_height, value
            "#,
            db_item.subject,
            db_item.producer_address,
            db_item.block_height,
            db_item.value
        )
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}

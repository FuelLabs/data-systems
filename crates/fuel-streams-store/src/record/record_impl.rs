use std::sync::Arc;

use async_trait::async_trait;
pub use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};
use fuel_streams_macros::subject::IntoSubject;

use super::{RecordEntity, RecordPacket};
use crate::db::{Db, DbError, DbItem, DbResult};

pub trait RecordEncoder: DataEncoder<Err = DbError> {}
impl<T: DataEncoder<Err = DbError>> RecordEncoder for T {}

#[async_trait]
pub trait Record: RecordEncoder + 'static {
    type DbItem: DbItem;

    const ENTITY: RecordEntity;
    const ORDER_PROPS: &'static [&'static str];

    fn to_packet(&self, subject: Arc<dyn IntoSubject>) -> RecordPacket<Self> {
        RecordPacket::new(subject, self)
    }

    async fn from_db_item(record: &Self::DbItem) -> DbResult<Self> {
        Self::decode(record.encoded_value()).await
    }

    async fn insert(
        &self,
        db: &Db,
        packet: &RecordPacket<Self>,
    ) -> DbResult<Self::DbItem>;

    async fn find_many_by_subject(
        db: &Db,
        subject: &Arc<dyn IntoSubject>,
        offset: i64,
        limit: i64,
        from_block: Option<u64>,
    ) -> DbResult<Vec<Self::DbItem>> {
        let sql_where = subject.to_sql_where();
        let sql_where = if let Some(from_block) = from_block {
            format!("{} AND block_height >= {}", sql_where, from_block)
        } else {
            sql_where
        };
        let query = format!(
            "SELECT * FROM {} WHERE {} ORDER BY {} DESC LIMIT {} OFFSET {}",
            Self::ENTITY.table_name(),
            sql_where,
            Self::ORDER_PROPS.join(", "),
            limit,
            offset
        );

        let records = sqlx::query_as::<_, Self::DbItem>(&query)
            .fetch_all(&db.pool)
            .await
            .map_err(DbError::FindManyByPattern)?;

        Ok(records)
    }

    async fn find_last_record(db: &Db) -> DbResult<Option<Self::DbItem>> {
        let query = format!(
            "SELECT * FROM {} ORDER BY {} DESC LIMIT 1",
            Self::ENTITY.table_name(),
            Self::ORDER_PROPS.join(", ")
        );

        let record = sqlx::query_as::<_, Self::DbItem>(&query)
            .fetch_optional(&db.pool)
            .await
            .map_err(DbError::FindManyByPattern)?;

        Ok(record)
    }

    /// TODO: Remove this once we have a better way to filter records by namespace
    /// This is a temporary solution to allow testing with namespaces
    #[cfg(any(test, feature = "test-helpers"))]
    async fn find_many_by_subject_ns(
        db: &Db,
        subject: &Arc<dyn IntoSubject>,
        namespace: &str,
        offset: i64,
        limit: i64,
        from_block: Option<u64>,
    ) -> DbResult<Vec<Self::DbItem>> {
        let records =
            Self::find_many_by_subject(db, subject, offset, limit, from_block)
                .await?;
        let records = records
            .into_iter()
            .filter(|record| record.subject_str().starts_with(namespace))
            .collect();
        Ok(records)
    }

    /// TODO: Remove this once we have a better way to filter records by namespace
    /// This is a temporary solution to allow testing with namespaces
    #[cfg(any(test, feature = "test-helpers"))]
    async fn find_last_record_ns(
        db: &Db,
        namespace: &str,
    ) -> DbResult<Option<Self::DbItem>> {
        let query = format!(
            "SELECT * FROM {} WHERE subject LIKE '{}%' ORDER BY {} DESC LIMIT 1",
            Self::ENTITY.table_name(),
            namespace,
            Self::ORDER_PROPS.join(", ")
        );

        let record = sqlx::query_as::<_, Self::DbItem>(&query)
            .fetch_optional(&db.pool)
            .await
            .map_err(DbError::FindManyByPattern)?;

        Ok(record)
    }
}

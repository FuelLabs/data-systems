use std::sync::Arc;

use async_trait::async_trait;
pub use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};
use fuel_streams_macros::subject::IntoSubject;

use super::{QueryOptions, RecordEntity, RecordPacket};
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
        options: QueryOptions,
    ) -> DbResult<Vec<Self::DbItem>> {
        let mut query_builder = sqlx::QueryBuilder::new(format!(
            "SELECT * FROM {}",
            Self::ENTITY.table_name()
        ));

        query_builder.push(" WHERE ").push(subject.to_sql_where());
        if let Some(block) = options.from_block {
            query_builder
                .push(" AND block_height >= ")
                .push_bind(block as i64);
        }

        query_builder
            .push(" ORDER BY ")
            .push(Self::ORDER_PROPS.join(", "))
            .push(" DESC LIMIT ")
            .push_bind(options.limit)
            .push(" OFFSET ")
            .push_bind(options.offset);

        let query = query_builder.build_query_as::<Self::DbItem>();
        let mut records = query
            .fetch_all(&db.pool)
            .await
            .map_err(DbError::FindManyByPattern)?;

        if cfg!(any(test, feature = "test-helpers")) {
            if let Some(ns) = options.namespace {
                records.retain(|record| record.subject_str().starts_with(&ns));
            }
        }

        Ok(records)
    }

    async fn find_last_record(
        db: &Db,
        namespace: Option<&str>,
    ) -> DbResult<Option<Self::DbItem>> {
        let mut query_builder = sqlx::QueryBuilder::new(format!(
            "SELECT * FROM {}",
            Self::ENTITY.table_name()
        ));

        if cfg!(any(test, feature = "test-helpers")) {
            if let Some(ns) = namespace {
                query_builder
                    .push(" WHERE subject LIKE ")
                    .push_bind(format!("{}%", ns));
            }
        }

        query_builder
            .push(" ORDER BY ")
            .push(Self::ORDER_PROPS.join(", "))
            .push(" DESC LIMIT 1");

        let query = query_builder.build_query_as::<Self::DbItem>();
        let record = query
            .fetch_optional(&db.pool)
            .await
            .map_err(DbError::FindManyByPattern)?;

        Ok(record)
    }
}

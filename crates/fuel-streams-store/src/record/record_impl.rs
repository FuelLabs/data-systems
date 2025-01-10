use std::sync::Arc;

use async_trait::async_trait;
pub use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};
use fuel_streams_macros::subject::IntoSubject;
use sqlx::{Execute, Postgres, QueryBuilder};

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

    fn build_find_many_query(
        subject: Arc<dyn IntoSubject>,
        options: QueryOptions,
    ) -> String {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        let select = format!("SELECT * FROM {}", Self::ENTITY.table_name());
        query_builder.push(select);
        query_builder.push(" WHERE ");
        query_builder.push(subject.to_sql_where());
        if let Some(block) = options.from_block {
            query_builder.push(" AND block_height >= ");
            query_builder.push_bind(block as i64);
        }
        query_builder.push(" ORDER BY ");
        query_builder.push(Self::ORDER_PROPS.join(", "));
        query_builder.push(" ASC LIMIT ");
        query_builder.push_bind(options.limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(options.offset);
        query_builder.build().sql().to_string()
    }

    async fn find_many_by_subject(
        db: &Db,
        subject: &Arc<dyn IntoSubject>,
        options: QueryOptions,
    ) -> DbResult<Vec<Self::DbItem>> {
        let query =
            Self::build_find_many_query(subject.clone(), options.clone());
        let mut records = sqlx::query_as::<_, Self::DbItem>(&query)
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

use std::sync::Arc;

use async_trait::async_trait;
pub use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};
use fuel_streams_macros::subject::IntoSubject;
use sqlx::{PgConnection, PgExecutor, Postgres, QueryBuilder};

use super::{QueryOptions, RecordEntity, RecordPacket};
use crate::db::{Db, DbError, DbItem, DbResult};

pub trait RecordEncoder: DataEncoder<Err = DbError> {}
impl<T: DataEncoder<Err = DbError>> RecordEncoder for T {}

pub type DbTransaction = sqlx::Transaction<'static, sqlx::Postgres>;
pub type DbConnection = PgConnection;

#[async_trait]
pub trait Record: RecordEncoder + 'static {
    type DbItem: DbItem;

    const ENTITY: RecordEntity;
    const ORDER_PROPS: &'static [&'static str];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>;

    async fn insert_with_transaction(
        tx: &mut DbTransaction,
        packet: &RecordPacket,
    ) -> DbResult<Self::DbItem> {
        Self::insert(&mut **tx, packet).await
    }

    fn to_packet(&self, subject: &Arc<dyn IntoSubject>) -> RecordPacket {
        let value = self
            .encode_json()
            .unwrap_or_else(|_| panic!("Encode failed for {}", Self::ENTITY));
        RecordPacket::new(subject.to_owned(), value)
    }

    async fn from_db_item(record: &Self::DbItem) -> DbResult<Self> {
        Self::decode(record.encoded_value()).await
    }

    fn build_find_many_query(
        subject: Arc<dyn IntoSubject>,
        options: QueryOptions,
    ) -> QueryBuilder<'static, Postgres> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        let select = format!("SELECT * FROM {}", Self::ENTITY.table_name());
        query_builder.push(select);
        query_builder.push(" WHERE ");
        query_builder.push(subject.to_sql_where());

        if let Some(block) = options.from_block {
            query_builder.push(" AND block_height >= ");
            query_builder.push_bind(block as i64);
        }

        if cfg!(any(test, feature = "test-helpers")) {
            if let Some(ns) = options.namespace {
                query_builder.push(" AND subject LIKE ");
                query_builder.push_bind(format!("{}%", ns));
            }
        }

        query_builder.push(" ORDER BY ");
        query_builder.push(Self::ORDER_PROPS.join(", "));
        query_builder.push(" ASC LIMIT ");
        query_builder.push_bind(options.limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(options.offset);
        query_builder
    }

    async fn find_last_record(
        db: &Db,
        options: QueryOptions,
    ) -> DbResult<Option<Self::DbItem>> {
        let mut query_builder = sqlx::QueryBuilder::new(format!(
            "SELECT * FROM {}",
            Self::ENTITY.table_name()
        ));

        if let Some(ns) = options.namespace {
            query_builder
                .push(" WHERE subject LIKE ")
                .push_bind(format!("{}%", ns));
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

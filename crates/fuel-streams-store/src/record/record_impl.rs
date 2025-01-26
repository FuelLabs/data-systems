use std::sync::Arc;

use async_trait::async_trait;
pub use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};
use fuel_streams_macros::subject::IntoSubject;
use sqlx::{PgConnection, PgExecutor, Postgres, QueryBuilder};

use super::{QueryOptions, RecordEntity, RecordPacket};
use crate::db::{DbError, DbItem, DbResult};

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

    fn from_db_item(record: &Self::DbItem) -> DbResult<Self> {
        Self::decode_json(record.encoded_value())
    }

    fn build_find_many_query(
        subject: Arc<dyn IntoSubject>,
        options: QueryOptions,
    ) -> QueryBuilder<'static, Postgres> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        let order_props = Self::ORDER_PROPS.join(", ");

        if order_props != "block_height" {
            query_builder.push("WITH items AS (");
        }

        // Internal select statement
        query_builder.push("SELECT ");
        if options.distinct {
            query_builder.push("DISTINCT ON (block_height) ");
        }
        query_builder.push("* FROM ");
        query_builder.push(Self::ENTITY.table_name());
        let mut conditions = Vec::new();
        if let Some(where_clause) = subject.to_sql_where() {
            conditions.push(where_clause);
        }
        if let Some(block) = options.from_block {
            conditions.push(format!("block_height >= {}", block));
        }
        if cfg!(any(test, feature = "test-helpers")) {
            if let Some(ns) = options.namespace {
                conditions.push(format!("subject LIKE '{ns}%'"));
            }
        }
        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }
        query_builder.push(" ORDER BY block_height ASC");
        query_builder.push(" LIMIT ");
        query_builder.push_bind(options.limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(options.offset);

        if order_props != "block_height" {
            query_builder.push(") SELECT * FROM items ");
            query_builder.push("ORDER BY ");
            query_builder.push(Self::ORDER_PROPS.join(", "));
            query_builder.push(" ASC");
        }

        tracing::debug!("Query built: {}", &query_builder.sql());
        query_builder
    }
}

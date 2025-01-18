use std::sync::Arc;

use async_trait::async_trait;
pub use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};
use fuel_streams_macros::subject::IntoSubject;
use sqlx::{PgConnection, PgExecutor, Postgres, QueryBuilder};

use super::{QueryOptions, RecordEntity, RecordPacket};
use crate::db::{Db, DbError, DbItem, DbResult, StoreItem};

pub trait RecordEncoder: DataEncoder<Err = DbError> {}
impl<T: DataEncoder<Err = DbError>> RecordEncoder for T {}

pub type DbTransaction = sqlx::Transaction<'static, sqlx::Postgres>;
pub type DbConnection = PgConnection;

#[async_trait]
pub trait Record: RecordEncoder + 'static {
    type DbItem: DbItem;
    type StoreItem: StoreItem;

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

    fn from_store_item(item: &Self::StoreItem) -> DbResult<Self> {
        Self::decode_json(item.encoded_value())
    }

    fn build_find_many_query(
        subject: Arc<dyn IntoSubject>,
        options: QueryOptions,
    ) -> QueryBuilder<'static, Postgres> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();

        query_builder.push("SELECT ");
        query_builder.push(Self::build_select_fields());
        query_builder.push(" FROM ");
        query_builder.push(Self::ENTITY.table_name());

        let mut conditions = Vec::new();

        // Add subject conditions if any
        if let Some(where_clause) = subject.to_sql_where() {
            conditions.push(where_clause);
        }

        // Add block conditions
        if let Some(block) = options.from_block {
            conditions.push(format!("block_height >= {}", block));
        }

        if let Some(block) = options.to_block {
            conditions.push(format!("block_height < {}", block));
        }

        // Add namespace condition for tests
        if cfg!(any(test, feature = "test-helpers")) {
            if let Some(ns) = options.namespace {
                conditions.push(format!("subject LIKE '{ns}%'"));
            }
        }

        // Add WHERE clause if we have any conditions
        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        query_builder.push(" ORDER BY ");
        query_builder.push(Self::ORDER_PROPS.join(", "));
        query_builder.push(" ASC LIMIT ");
        query_builder.push_bind(options.limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(options.offset);
        query_builder
    }

    fn build_select_fields() -> String {
        let mut select_fields = vec![
            "_id".to_string(),
            "subject".to_string(),
            "value".to_string(),
        ];

        // Add order fields first
        let order_fields: Vec<String> =
            Self::ORDER_PROPS.iter().map(|&s| s.to_string()).collect();
        select_fields.extend(order_fields);
        select_fields.join(", ")
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

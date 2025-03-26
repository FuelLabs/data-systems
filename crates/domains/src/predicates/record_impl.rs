use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{QueryOptions, Record, RecordEntity},
};
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

use super::{Predicate, PredicateDbItem};

#[async_trait]
impl Record for Predicate {
    type DbItem = PredicateDbItem;

    const ENTITY: RecordEntity = RecordEntity::Predicate;
    const ORDER_PROPS: &'static [&'static str] = &["tx_index", "input_index"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: Self::DbItem,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let mut tx = sqlx::Acquire::begin(executor)
            .await
            .map_err(DbError::Insert)?;

        let predicate_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO predicates (
                value,
                blob_id,
                predicate_address,
                created_at,
                published_at
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (predicate_address) DO UPDATE
            SET blob_id = EXCLUDED.blob_id,
                value = EXCLUDED.value,
                published_at = EXCLUDED.published_at
            RETURNING id",
        )
        .bind(db_item.value.clone())
        .bind(db_item.blob_id.clone())
        .bind(db_item.predicate_address.clone())
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(DbError::Insert)?;

        // Insert the transaction relationship
        sqlx::query(
            "INSERT INTO predicate_transactions (
                predicate_id,
                subject,
                block_height,
                tx_id,
                tx_index,
                input_index
            )
            VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(predicate_id)
        .bind(db_item.subject.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .execute(&mut *tx)
        .await
        .map_err(DbError::Insert)?;

        tx.commit().await.map_err(DbError::Insert)?;
        Ok(PredicateDbItem {
            published_at,
            ..db_item.to_owned()
        })
    }

    fn build_find_many_query(
        subject: std::sync::Arc<dyn IntoSubject>,
        options: QueryOptions,
    ) -> QueryBuilder<'static, Postgres> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT p.*, pt.subject, pt.block_height, pt.tx_id, pt.tx_index, pt.input_index
             FROM predicates p
             LEFT JOIN predicate_transactions pt ON p.id = pt.predicate_id"
        );

        let mut conditions = Vec::new();

        // Apply subject-based WHERE clause if provided
        if let Some(where_clause) = subject.to_sql_where() {
            conditions.push(where_clause);
        }

        // Apply block height filter from options
        if let Some(block) = options.from_block {
            conditions.push(format!("pt.block_height >= {}", block));
        }

        // Apply namespace filter if in test mode or test-helpers feature is enabled
        if cfg!(any(test, feature = "test-helpers")) {
            if let Some(ns) = options.namespace {
                conditions.push(format!("pt.subject LIKE '{ns}%'"));
            }
        }

        // Combine conditions with AND
        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        // Order by the specified ORDER_PROPS (block_height, tx_index, input_index)
        query_builder.push(" ORDER BY ");
        query_builder.push(Self::ORDER_PROPS.join(", "));
        query_builder.push(" ASC");

        // Apply pagination
        query_builder.push(" LIMIT ");
        query_builder.push_bind(options.limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(options.offset);

        tracing::info!("Query built: {}", &query_builder.sql());
        query_builder
    }
}

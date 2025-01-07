use async_trait::async_trait;
pub use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};

use super::{RecordEntity, RecordOrder};
use crate::{
    db::{Db, DbError, DbRecord, DbResult},
    store::StorePacket,
    subject_validator::SubjectValidator,
};

pub trait RecordEncoder: DataEncoder<Err = DbError> {}
impl<T: DataEncoder<Err = DbError>> RecordEncoder for T {}

#[async_trait]
pub trait Record: RecordEncoder + 'static {
    const ENTITY: RecordEntity;

    async fn insert(
        &self,
        db: &Db,
        subject: &str,
        order: RecordOrder,
    ) -> DbResult<DbRecord> {
        let entity = Self::ENTITY;
        let value = self.encode().await?;
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            INSERT INTO records (entity, order_block, order_tx, order_record, subject, value)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING entity as "entity: RecordEntity", order_block, order_tx, order_record, subject, value
            "#,
            entity as _,
            order.block,
            order.tx.unwrap_or(0),
            order.record.unwrap_or(0),
            subject,
            value,
        )
        .fetch_one(&db.pool)
        .await
        .map_err(|e| {
            if e.to_string()
                .contains("duplicate subject value violates unique constraint")
            {
                DbError::DuplicateSubject(subject.to_string())
            } else {
                DbError::Insert(e)
            }
        })?;

        Ok(record)
    }

    async fn update(
        &self,
        db: &Db,
        subject: &str,
        order: RecordOrder,
    ) -> DbResult<DbRecord> {
        let entity = Self::ENTITY;
        let value = self.encode().await?;
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            UPDATE records
            SET entity = $1, order_block = $2, order_tx = $3, order_record = $4, value = $5
            WHERE subject = $6
            RETURNING entity as "entity: RecordEntity", order_block, order_tx, order_record, subject, value
            "#,
            entity as _,
            order.block,
            order.tx.unwrap_or(0),
            order.record.unwrap_or(0),
            value,
            subject
        )
        .fetch_optional(&db.pool)
        .await
        .map_err(DbError::Update)?
        .ok_or_else(|| DbError::NotFound(subject.to_string()))?;

        Ok(record)
    }

    async fn upsert(
        &self,
        db: &Db,
        subject: &str,
        order: RecordOrder,
    ) -> DbResult<DbRecord> {
        let entity = Self::ENTITY;
        let value = self.encode().await?;
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            INSERT INTO records (entity, order_block, order_tx, order_record, subject, value)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (subject)
            DO UPDATE SET
                entity = EXCLUDED.entity,
                order_block = EXCLUDED.order_block,
                order_tx = EXCLUDED.order_tx,
                order_record = EXCLUDED.order_record,
                value = EXCLUDED.value
            RETURNING entity as "entity: RecordEntity", order_block, order_tx, order_record, subject, value
            "#,
            entity as _,
            order.block,
            order.tx.unwrap_or(0),
            order.record.unwrap_or(0),
            subject,
            value,
        )
        .fetch_one(&db.pool)
        .await
        .map_err(DbError::Upsert)?;

        Ok(record)
    }

    async fn delete(&self, db: &Db, subject: &str) -> DbResult<DbRecord> {
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            DELETE FROM records
            WHERE subject = $1
            RETURNING entity as "entity: RecordEntity", order_block, order_tx, order_record, subject, value
            "#,
            subject,
        )
        .fetch_optional(&db.pool)
        .await
        .map_err(DbError::Delete)?
        .ok_or_else(|| DbError::NotFound(subject.to_string()))?;

        Ok(record)
    }

    async fn find_many_by_pattern(
        db: &Db,
        pattern: &str,
    ) -> DbResult<Vec<DbRecord>> {
        let pg_pattern = SubjectValidator::to_sql_pattern(pattern);
        let records = if pattern.contains('>') {
            sqlx::query_as!(
                DbRecord,
                r#"SELECT entity as "entity: RecordEntity", subject, order_block, order_tx, order_record, value
                FROM records
                WHERE subject LIKE $1
                ORDER BY order_block, order_tx, order_record"#,
                pg_pattern
            )
                .fetch_all(&db.pool)
                .await
                .map_err(DbError::Query)?
        } else {
            sqlx::query_as!(
                DbRecord,
                r#"SELECT entity as "entity: RecordEntity", subject, order_block, order_tx, order_record, value
                FROM records
                WHERE subject ~ $1
                ORDER BY order_block, order_tx, order_record"#,
                pg_pattern
            )
                .fetch_all(&db.pool)
                .await
                .map_err(DbError::Query)?
        };

        Ok(records)
    }

    async fn find_last_record(db: &Db) -> DbResult<Option<DbRecord>> {
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            SELECT entity as "entity: RecordEntity", subject, order_block, order_tx, order_record, value
            FROM records
            ORDER BY order_block DESC, order_tx DESC, order_record DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&db.pool)
        .await
        .map_err(DbError::Query)?;

        Ok(record)
    }

    async fn from_db_record(record: &DbRecord) -> DbResult<Self> {
        Self::decode(&record.value).await
    }

    fn to_packet(
        &self,
        subject: impl Into<String>,
        order: &RecordOrder,
    ) -> StorePacket<Self> {
        StorePacket::new(self, subject.into(), order.clone())
    }
}

#[macro_export]
macro_rules! impl_record_for {
    ($type:ty, $entity:expr) => {
        use fuel_streams_store::{db::DbError, record::DataEncoder};

        impl DataEncoder for $type {
            type Err = DbError;
        }

        impl Record for $type {
            const ENTITY: RecordEntity = $entity;
        }
    };
}

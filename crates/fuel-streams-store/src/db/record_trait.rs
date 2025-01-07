use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use super::{Db, DbError, RecordEntity};
use crate::{store::StorePacket, subject_validator::SubjectValidator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbRecord {
    pub subject: String,
    pub entity: RecordEntity,
    pub sequence_order: i32,
    pub value: Vec<u8>,
}

pub type DbResult<T> = Result<T, DbError>;

#[async_trait]
pub trait Record:
    std::fmt::Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static
{
    const ENTITY: RecordEntity;

    async fn insert(
        &self,
        db: &Db,
        subject: &str,
        order: i32,
    ) -> DbResult<DbRecord> {
        let entity = Self::ENTITY;
        let value = self.encode();
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            INSERT INTO records (entity, sequence_order, subject, value)
            VALUES ($1, $2, $3, $4)
            RETURNING entity as "entity: RecordEntity", sequence_order, subject, value
            "#,
            entity as _,
            order,
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
        order: i32,
    ) -> DbResult<DbRecord> {
        let entity = Self::ENTITY;
        let value = self.encode();
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            UPDATE records
            SET entity = $1, sequence_order = $2, subject = $3, value = $4
            WHERE subject = $5
            RETURNING entity as "entity: RecordEntity", sequence_order, subject, value
            "#,
            entity as _,
            order,
            subject,
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
        order: i32,
    ) -> DbResult<DbRecord> {
        let entity = Self::ENTITY;
        let value = self.encode();
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            INSERT INTO records (entity, sequence_order, subject, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (subject)
            DO UPDATE SET
                entity = EXCLUDED.entity,
                sequence_order = EXCLUDED.sequence_order,
                value = EXCLUDED.value
            RETURNING entity as "entity: RecordEntity", sequence_order, subject, value
            "#,
            entity as _,
            order,
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
            RETURNING entity as "entity: RecordEntity", sequence_order, subject, value
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
                r#"SELECT entity as "entity: RecordEntity", subject, sequence_order, value FROM records WHERE subject LIKE $1"#,
                pg_pattern
            )
                .fetch_all(&db.pool)
                .await
                .map_err(DbError::Query)?
        } else {
            sqlx::query_as!(
                DbRecord,
                r#"SELECT entity as "entity: RecordEntity", subject, sequence_order, value FROM records WHERE subject ~ $1"#,
                pg_pattern
            )
                .fetch_all(&db.pool)
                .await
                .map_err(DbError::Query)?
        };

        Ok(records)
    }

    async fn find_last_record(db: &Db) -> DbResult<DbRecord> {
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            SELECT entity as "entity: RecordEntity", subject, sequence_order, value
            FROM records
            ORDER BY sequence_order DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&db.pool)
        .await
        .map_err(DbError::Query)?
        .ok_or_else(|| DbError::NotFound("No records found".to_string()))?;

        Ok(record)
    }

    fn encode(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn encode_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }

    fn decode(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }

    fn decode_json(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes).unwrap()
    }

    fn from_db_record(record: &DbRecord) -> Self {
        Self::decode(&record.value)
    }

    fn to_packet(&self, subject: impl Into<String>) -> StorePacket<Self> {
        StorePacket::new(&self, subject.into())
    }
}

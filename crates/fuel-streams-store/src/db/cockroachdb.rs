use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use super::{CockroachDbError, Db, DbRecord, DbResult};
use crate::subject_validator::SubjectValidator;

pub struct CockroachConnectionOpts {
    pub connection_str: String,
    pub pool_size: Option<u32>,
}
impl Default for CockroachConnectionOpts {
    fn default() -> Self {
        Self {
            pool_size: Some(5),
            connection_str: dotenvy::var("DATABASE_URL")
                .expect("DATABASE_URL not set"),
        }
    }
}

pub struct CockroachDb {
    pool: Pool<Postgres>,
}

impl CockroachDb {
    pub async fn new(opts: CockroachConnectionOpts) -> DbResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(opts.pool_size.unwrap_or_default())
            .connect(&opts.connection_str)
            .await
            .map_err(CockroachDbError::Open)?;

        Ok(Self { pool })
    }

    #[cfg(feature = "test-helpers")]
    pub async fn cleanup_tables(&self) -> DbResult<()> {
        sqlx::query("TRUNCATE TABLE records")
            .execute(&self.pool)
            .await
            .map_err(CockroachDbError::Delete)?;

        Ok(())
    }
}

#[async_trait]
impl Db for CockroachDb {
    async fn insert(&self, subject: &str, value: &[u8]) -> DbResult<DbRecord> {
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            INSERT INTO records (subject, value)
            VALUES ($1, $2)
            RETURNING subject, value
            "#,
            subject,
            value
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string()
                .contains("duplicate subject value violates unique constraint")
            {
                CockroachDbError::DuplicateSubject(subject.to_string())
            } else {
                CockroachDbError::Insert(e)
            }
        })?;

        Ok(record)
    }

    async fn update(&self, subject: &str, value: &[u8]) -> DbResult<DbRecord> {
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            UPDATE records
            SET value = $1
            WHERE subject = $2
            RETURNING subject, value
            "#,
            value,
            subject
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(CockroachDbError::Update)?
        .ok_or_else(|| CockroachDbError::NotFound(subject.to_string()))?;

        Ok(record)
    }

    async fn upsert(&self, subject: &str, value: &[u8]) -> DbResult<DbRecord> {
        let record = sqlx::query_as!(
            DbRecord,
            r#"
            INSERT INTO records (subject, value)
            VALUES ($1, $2)
            ON CONFLICT (subject) DO UPDATE
            SET value = EXCLUDED.value
            RETURNING subject, value
            "#,
            subject,
            value
        )
        .fetch_one(&self.pool)
        .await
        .map_err(CockroachDbError::Upsert)?;

        Ok(record)
    }

    async fn delete(&self, subject: &str) -> DbResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM records
            WHERE subject = $1
            "#,
            subject
        )
        .execute(&self.pool)
        .await
        .map_err(CockroachDbError::Delete)?;

        if result.rows_affected() == 0 {
            return Err(CockroachDbError::NotFound(subject.to_string()).into());
        }
        Ok(())
    }

    async fn find_by_pattern(&self, pattern: &str) -> DbResult<Vec<DbRecord>> {
        let pg_pattern = SubjectValidator::to_sql_pattern(pattern);
        let records = if pattern.contains('>') {
            sqlx::query_as!(
                DbRecord,
                r#"SELECT subject, value FROM records WHERE subject LIKE $1"#,
                pg_pattern
            )
            .fetch_all(&self.pool)
            .await
            .map_err(CockroachDbError::Query)?
        } else {
            sqlx::query_as!(
                DbRecord,
                r#"SELECT subject, value FROM records WHERE subject ~ $1"#,
                pg_pattern
            )
            .fetch_all(&self.pool)
            .await
            .map_err(CockroachDbError::Query)?
        };

        Ok(records)
    }
}

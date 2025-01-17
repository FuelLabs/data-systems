use std::sync::{Arc, LazyLock};

use sqlx::{Pool, Postgres};

use crate::record::{EncoderError, RecordPacketError};

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to open database")]
    Open(#[source] sqlx::Error),
    #[error("Failed to insert data")]
    Insert(#[source] sqlx::Error),
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Failed to find many records by pattern")]
    FindManyByPattern(#[source] sqlx::Error),
    #[error("Failed to encode/decode data")]
    EncodeDecode(#[from] EncoderError),
    #[error("Other error: {0}")]
    Other(String),
    #[error(transparent)]
    DbItemFromPacket(#[from] RecordPacketError),
    #[error("Failed to truncate table")]
    TruncateTable(#[source] sqlx::Error),
    #[error("Failed to execute query")]
    Query(#[source] sqlx::Error),
    #[error("Failed to start database transaction: {0}")]
    BeginTransaction(#[source] sqlx::Error),
    #[error("Failed to commit transaction: {0}")]
    CommitTransaction(#[source] sqlx::Error),
}

pub type DbResult<T> = Result<T, DbError>;

pub static DB_POOL_SIZE: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("DB_POOL_SIZE")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(5)
});

#[derive(Debug, Clone)]
pub struct DbConnectionOpts {
    pub connection_str: String,
    pub pool_size: Option<u32>,
}

impl Default for DbConnectionOpts {
    fn default() -> Self {
        Self {
            pool_size: Some(*DB_POOL_SIZE as u32),
            connection_str: dotenvy::var("DATABASE_URL")
                .expect("DATABASE_URL not set"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    pub async fn new(opts: DbConnectionOpts) -> DbResult<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(opts.pool_size.unwrap_or_default())
            .connect(&opts.connection_str)
            .await
            .map_err(DbError::Open)?;

        tracing::info!("Database connected");
        Ok(Self { pool })
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub async fn truncate_table(&self, table_name: &str) -> DbResult<()> {
        let query = format!("TRUNCATE TABLE {}", table_name);
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(DbError::TruncateTable)?;
        Ok(())
    }

    pub fn pool_ref(&self) -> &Pool<Postgres> {
        &self.pool
    }

    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'static, sqlx::Postgres>, DbError> {
        self.pool.begin().await.map_err(DbError::BeginTransaction)
    }

    pub async fn commit_transaction(
        &self,
        transaction: sqlx::Transaction<'static, sqlx::Postgres>,
    ) -> Result<(), DbError> {
        transaction
            .commit()
            .await
            .map_err(DbError::CommitTransaction)
    }
}

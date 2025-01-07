use std::sync::Arc;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::record::{EncoderError, RecordEntity};

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to open database")]
    Open(#[source] sqlx::Error),
    #[error("Failed to insert data")]
    Insert(#[source] sqlx::Error),
    #[error("Duplicate subject: {0}")]
    DuplicateSubject(String),
    #[error("Failed to update data")]
    Update(#[source] sqlx::Error),
    #[error("Failed to upsert data")]
    Upsert(#[source] sqlx::Error),
    #[error("Failed to delete data")]
    Delete(#[source] sqlx::Error),
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Failed to find record")]
    Find(#[source] sqlx::Error),
    #[error("Failed to query data")]
    Query(#[source] sqlx::Error),
    #[error("Failed to encode/decode data")]
    EncodeDecode(#[from] EncoderError),
}

pub type OrderIntSize = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbRecord {
    pub subject: String,
    pub entity: RecordEntity,
    pub order_block: OrderIntSize,
    pub order_tx: Option<OrderIntSize>,
    pub order_record: Option<OrderIntSize>,
    pub value: Vec<u8>,
}

pub type DbResult<T> = Result<T, DbError>;

pub struct DbConnectionOpts {
    pub connection_str: String,
    pub pool_size: Option<u32>,
}
impl Default for DbConnectionOpts {
    fn default() -> Self {
        Self {
            pool_size: Some(5),
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
        let pool = PgPoolOptions::new()
            .max_connections(opts.pool_size.unwrap_or_default())
            .connect(&opts.connection_str)
            .await
            .map_err(DbError::Open)?;

        Ok(Self { pool })
    }

    #[cfg(feature = "test-helpers")]
    pub async fn cleanup_tables(&self) -> DbResult<()> {
        sqlx::query("TRUNCATE TABLE records")
            .execute(&self.pool)
            .await
            .map_err(DbError::Delete)?;

        Ok(())
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

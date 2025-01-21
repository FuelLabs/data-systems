use std::{
    str::FromStr,
    sync::{Arc, LazyLock},
    time::Duration,
};

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
pub type SqlxError = sqlx::Error;

pub static DB_POOL_SIZE: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("DB_POOL_SIZE")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(100)
});

#[derive(Debug, Clone)]
pub struct DbConnectionOpts {
    pub connection_str: String,
    pub pool_size: Option<u32>,
    pub statement_timeout: Option<Duration>,
    pub acquire_timeout: Option<Duration>,
    pub idle_timeout: Option<Duration>,
    pub min_connections: Option<u32>,
}

impl Default for DbConnectionOpts {
    fn default() -> Self {
        Self {
            pool_size: Some(*DB_POOL_SIZE as u32),
            connection_str: dotenvy::var("DATABASE_URL")
                .expect("DATABASE_URL not set"),
            statement_timeout: Some(Duration::from_secs(30)),
            acquire_timeout: Some(Duration::from_secs(10)),
            idle_timeout: Some(Duration::from_secs(180)),
            min_connections: Some((*DB_POOL_SIZE as u32) / 4),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    pub async fn new(opts: DbConnectionOpts) -> DbResult<Self> {
        let pool = Self::create_pool(&opts).await?;
        tracing::info!(
            "Database connected with pool size: {}",
            opts.pool_size.unwrap_or_default()
        );
        Ok(Self { pool })
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn pool_ref(&self) -> &Pool<Postgres> {
        &self.pool
    }

    async fn create_pool(opts: &DbConnectionOpts) -> DbResult<Pool<Postgres>> {
        let statement_timeout =
            opts.statement_timeout.unwrap_or_default().as_millis();
        let statement_timeout = &format!("{}", statement_timeout);
        let connections_opts =
            sqlx::postgres::PgConnectOptions::from_str(&opts.connection_str)
                .map_err(|e| {
                    DbError::Open(sqlx::Error::Configuration(Box::new(e)))
                })?
                .application_name("fuel-streams")
                .options([
                    ("retry_connect_backoff", "2"),
                    ("statement_timeout", statement_timeout),
                ]);

        sqlx::postgres::PgPoolOptions::new()
            .max_connections(opts.pool_size.unwrap_or_default())
            .min_connections(opts.min_connections.unwrap_or_default())
            .acquire_timeout(opts.acquire_timeout.unwrap_or_default())
            .idle_timeout(opts.idle_timeout)
            .test_before_acquire(false)
            .connect_with(connections_opts)
            .await
            .map_err(DbError::Open)
    }
}

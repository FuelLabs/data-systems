use std::{
    str::FromStr,
    sync::{Arc, LazyLock},
    time::Duration,
};

use fuel_data_parser::DataParserError;
use hex::FromHexError;
use sqlx::{Pool, Postgres};

use crate::infra::RecordPacketError;

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
    DataParser(#[from] DataParserError),
    #[error("Other error: {0}")]
    Other(String),
    #[error("Failed to truncate table")]
    TruncateTable(#[source] sqlx::Error),
    #[error("Failed to execute query")]
    Query(#[source] sqlx::Error),
    #[error("Failed to start database transaction: {0}")]
    BeginTransaction(#[source] sqlx::Error),
    #[error("Failed to commit transaction: {0}")]
    CommitTransaction(#[source] sqlx::Error),
    #[error(transparent)]
    DbItemFromPacket(#[from] RecordPacketError),
    #[error(transparent)]
    Hex(#[from] FromHexError),
}

pub type DbTransaction = sqlx::Transaction<'static, sqlx::Postgres>;
pub type DbResult<T> = Result<T, DbError>;
pub type SqlxError = sqlx::Error;

pub static DB_POOL_SIZE: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("DB_POOL_SIZE")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(110)
});

pub static DB_ACQUIRE_TIMEOUT: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("DB_ACQUIRE_TIMEOUT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(180)
});

#[derive(Debug, Clone)]
pub struct DbConnectionOpts {
    pub connection_str: String,
    pub pool_size: Option<u32>,
    pub min_connections: Option<u32>,
    pub statement_timeout: Option<Duration>,
    pub acquire_timeout: Option<Duration>,
    pub idle_timeout: Option<Duration>,
}

impl Default for DbConnectionOpts {
    fn default() -> Self {
        Self {
            pool_size: Some(*DB_POOL_SIZE as u32),
            min_connections: Some(2),
            connection_str: dotenvy::var("DATABASE_URL")
                .expect("DATABASE_URL not set"),
            statement_timeout: Some(Duration::from_secs(240)),
            acquire_timeout: Some(Duration::from_secs(
                *DB_ACQUIRE_TIMEOUT as u64,
            )),
            idle_timeout: Some(Duration::from_secs(240)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    pub async fn new(opts: DbConnectionOpts) -> DbResult<Arc<Self>> {
        let pool = Self::create_pool(&opts).await?;
        Ok(Arc::new(Self { pool }))
    }

    pub fn pool_ref(&self) -> &Pool<Postgres> {
        &self.pool
    }

    async fn create_pool(opts: &DbConnectionOpts) -> DbResult<Pool<Postgres>> {
        let connections_opts =
            sqlx::postgres::PgConnectOptions::from_str(&opts.connection_str)
                .map_err(|e| {
                    DbError::Open(sqlx::Error::Configuration(Box::new(e)))
                })?
                .application_name("fuel-streams")
                .options(Self::connect_opts(opts));

        sqlx::postgres::PgPoolOptions::new()
            .min_connections(opts.min_connections.unwrap_or_default())
            .max_connections(opts.pool_size.unwrap_or_default())
            .acquire_timeout(opts.acquire_timeout.unwrap_or_default())
            .idle_timeout(opts.idle_timeout)
            .test_before_acquire(false)
            .connect_with(connections_opts)
            .await
            .map_err(DbError::Open)
    }

    fn connect_opts(opts: &DbConnectionOpts) -> Vec<(String, String)> {
        let statement_timeout =
            opts.statement_timeout.unwrap_or_default().as_millis();
        let statement_timeout = format!("{}", statement_timeout);
        let idle_timeout = opts.idle_timeout.unwrap_or_default().as_millis();
        let idle_timeout = format!("{}", idle_timeout);
        vec![
            ("statement_timeout".to_string(), statement_timeout),
            (
                "idle_in_transaction_session_timeout".to_string(),
                idle_timeout,
            ),
        ]
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

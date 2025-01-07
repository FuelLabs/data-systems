use std::sync::Arc;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use super::{DbError, DbResult};

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

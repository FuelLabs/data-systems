use async_trait::async_trait;
use fuel_streams_store::db::DbItem;

#[async_trait]
pub trait Queryable {
    type Record: DbItem;

    fn query_to_string(&self) -> String;

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<Self::Record>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>;
}

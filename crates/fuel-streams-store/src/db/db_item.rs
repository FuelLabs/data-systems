use async_trait::async_trait;
use fuel_data_parser::DataEncoder;
use sqlx::postgres::PgRow;

use super::DbError;
use crate::record::RecordEntity;

#[async_trait]
pub trait DbItem:
    DataEncoder<Err = DbError>
    + Unpin
    + std::fmt::Debug
    + PartialEq
    + Eq
    + Send
    + Sync
    + Sized
    + serde::Serialize
    + serde::de::DeserializeOwned
    + for<'r> sqlx::FromRow<'r, PgRow>
    + 'static
{
    fn entity(&self) -> &RecordEntity;
    fn encoded_value(&self) -> &[u8];
    fn subject_str(&self) -> String;
    fn get_block_height(&self) -> u64;
}

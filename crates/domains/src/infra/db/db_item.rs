use async_trait::async_trait;
use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use sqlx::postgres::PgRow;

use super::{Cursor, DbError};
use crate::infra::RecordEntity;

#[async_trait]
pub trait DbItem:
    DataEncoder
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
    fn cursor(&self) -> Cursor;
    fn entity(&self) -> &RecordEntity;
    fn encoded_value(&self) -> Result<Vec<u8>, DbError>;
    fn subject_str(&self) -> String;
    fn subject_id(&self) -> String;
    fn created_at(&self) -> BlockTimestamp;
    fn published_at(&self) -> BlockTimestamp;
    fn block_height(&self) -> BlockHeight;
}

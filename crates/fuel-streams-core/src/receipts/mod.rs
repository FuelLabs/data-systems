pub mod subjects;
pub mod types;

use fuel_streams_store::db::{Record, RecordEntity};
pub use subjects::*;

use super::types::*;

impl Record for Receipt {
    const ENTITY: RecordEntity = RecordEntity::Receipt;
}

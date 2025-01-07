pub mod subjects;
pub mod types;

use fuel_streams_store::{
    impl_record_for,
    record::{Record, RecordEntity},
};
pub use subjects::*;

use super::types::*;

impl_record_for!(Log, RecordEntity::Log);

pub mod types;

use fuel_streams_store::{
    impl_record_for,
    record::{Record, RecordEntity},
};
pub(crate) use types::*;

impl_record_for!(Utxo, RecordEntity::Utxo);

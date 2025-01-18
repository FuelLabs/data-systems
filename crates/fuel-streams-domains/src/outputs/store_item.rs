use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, StoreItem},
    record::{DataEncoder, RecordEntity},
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct OutputStoreItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_index: i64,
    pub output_index: i64,
}

impl DataEncoder for OutputStoreItem {
    type Err = DbError;
}

impl StoreItem for OutputStoreItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Output
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }
}

impl PartialOrd for OutputStoreItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OutputStoreItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
            // Finally by output index within the transaction
            .then(self.output_index.cmp(&other.output_index))
    }
}

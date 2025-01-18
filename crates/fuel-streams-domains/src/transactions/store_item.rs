use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, StoreItem},
    record::{DataEncoder, RecordEntity},
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct TransactionStoreItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_index: i64,
}

impl DataEncoder for TransactionStoreItem {
    type Err = DbError;
}

impl StoreItem for TransactionStoreItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Transaction
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }
}

impl PartialOrd for TransactionStoreItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransactionStoreItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
    }
}

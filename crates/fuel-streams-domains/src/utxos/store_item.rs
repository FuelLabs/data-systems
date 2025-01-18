use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, StoreItem},
    record::{DataEncoder, RecordEntity},
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct UtxoStoreItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub tx_index: i64,
    pub input_index: i64,
}

impl DataEncoder for UtxoStoreItem {
    type Err = DbError;
}

impl StoreItem for UtxoStoreItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Utxo
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }
}

impl PartialOrd for UtxoStoreItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UtxoStoreItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by block height first
        self.block_height
            .cmp(&other.block_height)
            // Then by transaction index within the block
            .then(self.tx_index.cmp(&other.tx_index))
            // Finally by input index
            .then(self.input_index.cmp(&other.input_index))
    }
}

use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, StoreItem},
    record::{DataEncoder, RecordEntity},
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct BlockStoreItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64, // This is our order prop
}

impl DataEncoder for BlockStoreItem {
    type Err = DbError;
}

impl StoreItem for BlockStoreItem {
    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Block
    }

    fn encoded_value(&self) -> &[u8] {
        &self.value
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }
}

impl PartialOrd for BlockStoreItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockStoreItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_height.cmp(&other.block_height)
    }
}

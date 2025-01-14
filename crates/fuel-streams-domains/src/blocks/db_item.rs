use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
};
use serde::{Deserialize, Serialize};

use super::Block;
use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct BlockDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub producer_address: String,
}

impl DataEncoder for BlockDbItem {
    type Err = DbError;
}

impl DbItem for BlockDbItem {
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

impl PartialOrd for BlockDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_height.cmp(&other.block_height)
    }
}

impl TryFrom<&RecordPacket<Block>> for BlockDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket<Block>) -> Result<Self, Self::Error> {
        let record = packet.record.as_ref();
        let subject: Subjects = packet
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::Block(_) => Ok(BlockDbItem {
                subject: packet.subject_str(),
                value: record.encode_json().expect("Failed to encode block"),
                block_height: record.height.clone().into(),
                producer_address: record.producer.to_string(),
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}

use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{
        DataEncoder,
        RecordEntity,
        RecordPacket,
        RecordPacketError,
        RecordPointer,
    },
};
use serde::{Deserialize, Serialize};

use super::{Block, BlockTimestamp, BlocksSubject};
use crate::Subjects;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct BlockDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: i64,
    pub producer_address: String,
    pub timestamp: DateTime<Utc>,
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

    fn subject_id(&self) -> String {
        BlocksSubject::ID.to_string()
    }
}

impl TryFrom<&RecordPacket> for BlockDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        let block =
            Block::decode_json(&packet.value).expect("Failed to decode block");
        let timestamp = BlockTimestamp::from(&block);
        match subject {
            Subjects::Block(subject) => Ok(BlockDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_height: subject.height.unwrap().into(),
                producer_address: subject.producer.unwrap().to_string(),
                timestamp: timestamp.into_inner(),
            }),
            _ => Err(RecordPacketError::SubjectMismatch),
        }
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

impl From<BlockDbItem> for RecordPointer {
    fn from(val: BlockDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height.into(),
            tx_index: None,
            input_index: None,
            output_index: None,
            receipt_index: None,
        }
    }
}

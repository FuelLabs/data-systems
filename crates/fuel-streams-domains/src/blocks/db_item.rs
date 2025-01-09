use std::cmp::Ordering;

use fuel_streams_store::{
    db::{DbError, DbItem},
    record::{DataEncoder, RecordEntity, RecordPacket, RecordPacketError},
    try_packet_subject_match,
};
use serde::{Deserialize, Serialize};

use super::{Block, BlocksSubject};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct BlockDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub height: i64,
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
        self.height.cmp(&other.height)
    }
}

impl TryFrom<&RecordPacket<Block>> for BlockDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket<Block>) -> Result<Self, Self::Error> {
        let record = packet.record.as_ref();
        try_packet_subject_match!(packet, {
            BlocksSubject => _subject => {
                Ok(BlockDbItem {
                    subject: packet.subject_str(),
                    value: record.encode_json().expect("Failed to encode block"),
                    height: record.height.clone().into(),
                    producer_address: record.producer.to_string(),
                })
            }
        })
    }
}

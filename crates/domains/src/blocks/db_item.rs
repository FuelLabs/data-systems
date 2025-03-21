use std::cmp::Ordering;

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
use fuel_streams_types::{
    wrapped_int::WrappedU32,
    BlockHeight,
    BlockTimestamp,
    DaBlockHeight,
};
use serde::{Deserialize, Serialize};

use super::BlocksSubject;
use crate::Subjects;
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct BlockDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub producer_address: String,
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
    pub block_propagation_ms: i32,

    pub header_application_hash: String,
    pub header_consensus_parameters_version: WrappedU32,
    pub header_da_height: DaBlockHeight,
    pub header_event_inbox_root: String,
    pub header_message_outbox_root: String,
    pub header_message_receipt_count: WrappedU32,
    pub header_prev_root: String,
    pub header_state_transition_bytecode_version: WrappedU32,
    pub header_time: BlockTimestamp,
    pub header_transactions_count: i32,
    pub header_transactions_root: String,
    pub header_version: i32,

    pub chain_config_hash: String,
    pub coins_root: String,
    pub contracts_root: String,
    pub messages_root: String,
    pub transactions_root: String,
    pub signature: String,
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

    fn created_at(&self) -> BlockTimestamp {
        self.created_at
    }

    fn published_at(&self) -> BlockTimestamp {
        self.published_at
    }

    fn block_height(&self) -> BlockHeight {
        self.block_height
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
        match subject {
            Subjects::Block(subject) => Ok(BlockDbItem {
                subject: packet.subject_str(),
                value: packet.value.to_owned(),
                block_da_height: subject.da_height.unwrap(),
                block_height: subject.height.unwrap(),
                producer_address: subject.producer.unwrap().to_string(),
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
                block_propagation_ms: 0,
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
            block_height: val.block_height,
            tx_index: None,
            input_index: None,
            output_index: None,
            receipt_index: None,
        }
    }
}

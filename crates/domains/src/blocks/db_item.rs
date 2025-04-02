use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::{
    wrapped_int::WrappedU32,
    BlockHeight,
    BlockTimestamp,
    DaBlockHeight,
    DbConsensusType,
};
use serde::{Deserialize, Serialize};

use super::{Block, BlocksSubject};
use crate::{
    infra::{
        db::DbItem,
        record::{
            RecordEntity,
            RecordPacket,
            RecordPacketError,
            RecordPointer,
        },
        Cursor,
        DbError,
    },
    Subjects,
};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow, Default,
)]
pub struct BlockDbItem {
    pub subject: String,
    pub block_height: BlockHeight,
    pub block_da_height: DaBlockHeight,
    pub value: Vec<u8>,
    pub version: String,
    pub producer_address: String,
    // timestamps
    pub created_at: BlockTimestamp,
    pub published_at: BlockTimestamp,
    pub block_propagation_ms: i32,
    // header
    pub header_application_hash: String,
    pub header_consensus_parameters_version: WrappedU32,
    pub header_da_height: DaBlockHeight,
    pub header_event_inbox_root: String,
    pub header_message_outbox_root: String,
    pub header_message_receipt_count: WrappedU32,
    pub header_prev_root: String,
    pub header_state_transition_bytecode_version: WrappedU32,
    pub header_time: BlockTimestamp,
    pub header_transactions_count: i16,
    pub header_transactions_root: String,
    pub header_version: String,
    // consensus
    pub consensus_chain_config_hash: Option<String>,
    pub consensus_coins_root: Option<String>,
    pub consensus_type: DbConsensusType,
    pub consensus_contracts_root: Option<String>,
    pub consensus_messages_root: Option<String>,
    pub consensus_signature: Option<String>,
    pub consensus_transactions_root: Option<String>,
}

impl DataEncoder for BlockDbItem {}

impl DbItem for BlockDbItem {
    fn cursor(&self) -> crate::infra::Cursor {
        Cursor::new(&[&self.block_height])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Block
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.clone())
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
        let block = Block::decode_json(&packet.value)
            .map_err(|e| RecordPacketError::DecodeFailed(e.to_string()))?;
        let (
            consensus_type,
            consensus_chain_config_hash,
            consensus_coins_root,
            consensus_contracts_root,
            consensus_messages_root,
            consensus_transactions_root,
            consensus_signature,
        ) = block.consensus.normalize_all();

        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::Block(subject) => Ok(BlockDbItem {
                subject: packet.subject_str(),
                block_height: subject.height.unwrap(),
                block_da_height: subject.da_height.unwrap(),
                value: packet.value.to_owned(),
                version: block.header.version.to_string(),
                producer_address: subject.producer.unwrap().to_string(),
                // timestamps
                created_at: packet.block_timestamp,
                published_at: packet.block_timestamp,
                block_propagation_ms: 0,
                // header
                header_application_hash: block
                    .header
                    .application_hash
                    .to_string(),
                header_consensus_parameters_version: block
                    .header
                    .consensus_parameters_version
                    .to_owned(),
                header_da_height: subject.da_height.unwrap(),
                header_event_inbox_root: block
                    .header
                    .event_inbox_root
                    .to_string(),
                header_message_outbox_root: block
                    .header
                    .message_outbox_root
                    .to_string(),
                header_message_receipt_count: block
                    .header
                    .message_receipt_count
                    .to_owned(),
                header_prev_root: block.header.prev_root.to_string(),
                header_state_transition_bytecode_version: block
                    .header
                    .state_transition_bytecode_version
                    .to_owned(),
                header_time: (&block.header).into(),
                header_transactions_count: block.header.transactions_count
                    as i16,
                header_transactions_root: block
                    .header
                    .transactions_root
                    .to_string(),
                header_version: block.header.version.to_string(),
                // consensus
                consensus_chain_config_hash: consensus_chain_config_hash
                    .map(|val| val.to_string()),
                consensus_coins_root: consensus_coins_root
                    .map(|val| val.to_string()),
                consensus_type: consensus_type.unwrap(),
                consensus_contracts_root: consensus_contracts_root
                    .map(|val| val.to_string()),
                consensus_messages_root: consensus_messages_root
                    .map(|val| val.to_string()),
                consensus_transactions_root: consensus_transactions_root
                    .map(|val| val.to_string()),
                consensus_signature: consensus_signature
                    .map(|val| val.to_string()),
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

impl TryFrom<&Block> for BlockDbItem {
    type Error = DbError;
    fn try_from(value: &Block) -> Result<Self, Self::Error> {
        let subject = BlocksSubject::from(value);
        let encoded = value.encode_json()?;
        Ok(Self {
            subject: subject.to_string(),
            value: encoded,
            block_da_height: subject.da_height.unwrap(),
            block_height: subject.height.unwrap(),
            producer_address: subject.producer.unwrap().to_string(),
            ..Default::default()
        })
    }
}

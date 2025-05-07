use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{Message, MessageType, MessagesSubject};
use crate::infra::{
    db::DbItem,
    record::{RecordEntity, RecordPacket, RecordPacketError, RecordPointer},
    Cursor,
    DbError,
};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow, Default,
)]
pub struct MessageDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub message_index: i32,
    pub cursor: String,
    // fields matching fuel-core
    pub r#type: MessageType,
    pub sender: String,
    pub recipient: String,
    pub nonce: String,
    pub amount: i64,
    pub data: String,
    pub da_height: DaBlockHeight,
    // timestamps
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl DataEncoder for MessageDbItem {}

impl DbItem for MessageDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.message_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Message
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.clone())
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        MessagesSubject::ID.to_string()
    }

    fn created_at(&self) -> BlockTimestamp {
        self.created_at
    }

    fn block_time(&self) -> BlockTimestamp {
        self.block_time
    }

    fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}

impl TryFrom<&RecordPacket> for MessageDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let message = Message::decode_json(&packet.value)?;
        let block_height = packet.pointer.block_height;
        let msg_index = message.message_index as i32;
        Ok(MessageDbItem {
            subject: packet.subject_str(),
            value: packet.value.to_owned(),
            block_height: packet.pointer.block_height,
            message_index: msg_index,
            cursor: format!("{}-{}", block_height, msg_index),
            r#type: message.r#type,
            sender: message.sender.to_string(),
            recipient: message.recipient.to_string(),
            nonce: message.nonce.to_string(),
            amount: message.amount.into_inner() as i64,
            data: message.data.to_string(),
            da_height: message.da_height,
            block_time: packet.block_timestamp,
            created_at: packet.block_timestamp,
        })
    }
}

impl PartialOrd for MessageDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessageDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_height
            .cmp(&other.block_height)
            .then(self.message_index.cmp(&other.message_index))
    }
}

impl From<MessageDbItem> for RecordPointer {
    fn from(val: MessageDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            ..Default::default()
        }
    }
}

use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{MessageType, MessagesQuery};
use crate::{
    infra::{QueryOptions, QueryPagination},
    messages::types::Message,
};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "messages")]
#[subject(entity = "Message")]
#[subject(query_all = "messages.>")]
#[subject(
    format = "messages.{message_type}.{block_height}.{message_index}.{sender}.{recipient}.{nonce}"
)]
pub struct MessagesSubject {
    #[subject(description = "The type of message (imported or consumed)")]
    pub message_type: Option<MessageType>,
    #[subject(description = "The height of the block containing this message")]
    pub block_height: Option<BlockHeight>,
    #[subject(description = "The index of the message within the block")]
    pub message_index: Option<i32>,
    #[subject(
        description = "The address that sent the message (32 byte string prefixed by 0x)"
    )]
    pub sender: Option<Address>,
    #[subject(
        description = "The address that will receive the message (32 byte string prefixed by 0x)"
    )]
    pub recipient: Option<Address>,
    #[subject(
        description = "The nonce of the message (32 byte string prefixed by 0x)"
    )]
    pub nonce: Option<Nonce>,
}

impl From<&Message> for MessagesSubject {
    fn from(message: &Message) -> Self {
        let subject = MessagesSubject::new();
        subject
            .with_message_type(Some(message.r#type))
            .with_block_height(Some(message.block_height))
            .with_message_index(Some(message.message_index as i32))
            .with_sender(Some(message.sender.clone()))
            .with_recipient(Some(message.recipient.clone()))
            .with_nonce(Some(message.nonce.clone()))
    }
}

impl From<MessagesSubject> for MessagesQuery {
    fn from(subject: MessagesSubject) -> Self {
        Self {
            block_height: subject.block_height,
            message_index: subject.message_index,
            message_type: subject.message_type,
            sender: subject.sender,
            recipient: subject.recipient,
            nonce: subject.nonce,
            da_height: None,
            address: None,
            pagination: QueryPagination::default(),
            options: QueryOptions::default(),
        }
    }
}

use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use rayon::prelude::*;

use super::{subjects::*, types::*, MessagesQuery};
use crate::{
    blocks::BlockHeight,
    infra::{
        record::{PacketBuilder, RecordPacket, ToPacket},
        RecordPointer,
    },
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Message {
    type Opts = MsgPayload;

    fn build_packets(msg_payload: &Self::Opts) -> Vec<RecordPacket> {
        msg_payload
            .events
            .par_iter()
            .enumerate()
            .filter_map(|(message_index, event)| {
                let message = Message::new(
                    msg_payload.block_height(),
                    message_index as u32,
                    event.clone(),
                )?;

                let subject = DynMessageSubject::new(
                    &message,
                    msg_payload.block_height(),
                    message_index as i32,
                );

                let timestamps = msg_payload.timestamp();
                let pointer = RecordPointer {
                    block_height: msg_payload.block_height(),
                    ..Default::default()
                };

                let packet =
                    subject.build_packet(&message, timestamps, pointer);
                Some(match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                })
            })
            .collect()
    }
}

pub enum DynMessageSubject {
    Message(MessagesSubject),
}

impl DynMessageSubject {
    pub fn new(
        message: &Message,
        block_height: BlockHeight,
        message_index: i32,
    ) -> Self {
        Self::Message(MessagesSubject {
            message_type: Some(message.r#type),
            block_height: Some(block_height),
            message_index: Some(message_index),
            sender: Some(message.sender.clone()),
            recipient: Some(message.recipient.clone()),
            nonce: Some(message.nonce.clone()),
        })
    }

    pub fn build_packet(
        &self,
        message: &Message,
        block_timestamp: BlockTimestamp,
        pointer: RecordPointer,
    ) -> RecordPacket {
        match self {
            Self::Message(subject) => message.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
        }
    }

    pub fn to_query_params(&self) -> MessagesQuery {
        match self {
            Self::Message(subject) => MessagesQuery::from(subject.to_owned()),
        }
    }
}

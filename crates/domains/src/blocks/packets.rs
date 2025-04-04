use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_types::{Address, BlockTimestamp, DaBlockHeight};

use super::{Block, BlocksQuery, BlocksSubject};
use crate::{
    blocks::BlockHeight,
    infra::{
        record::{PacketBuilder, RecordPacket, ToPacket},
        RecordPointer,
    },
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Block {
    type Opts = MsgPayload;
    fn build_packets(msg_payload: &Self::Opts) -> Vec<RecordPacket> {
        let block = msg_payload.block();
        let block_height = *msg_payload.metadata.block_height;
        let block_producer = (*msg_payload.metadata.block_producer).clone();
        let timestamps = msg_payload.timestamp();
        let subject = DynBlockSubject::new(
            block_height,
            block_producer,
            &block.header.da_height,
        );

        let packet = subject.build_packet(block, timestamps);
        let packet = match msg_payload.namespace.clone() {
            Some(ns) => packet.with_namespace(&ns),
            _ => packet,
        };

        std::iter::once(packet).collect()
    }
}

#[derive(Debug, Clone)]
pub enum DynBlockSubject {
    Block(BlocksSubject),
}

impl DynBlockSubject {
    pub fn new(
        block_height: BlockHeight,
        block_producer: Address,
        da_height: &DaBlockHeight,
    ) -> Self {
        Self::Block(BlocksSubject {
            producer: Some(block_producer),
            da_height: Some(da_height.to_owned()),
            height: Some(block_height),
        })
    }

    pub fn build_packet(
        &self,
        block: &Block,
        block_timestamp: BlockTimestamp,
    ) -> RecordPacket {
        match self {
            Self::Block(subject) => block.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                RecordPointer {
                    block_height: block.height,
                    ..Default::default()
                },
            ),
        }
    }

    pub fn to_query_params(&self) -> BlocksQuery {
        match self {
            Self::Block(subject) => BlocksQuery::from(subject.to_owned()),
        }
    }
}

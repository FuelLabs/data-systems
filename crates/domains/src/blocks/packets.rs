use async_trait::async_trait;

use super::{Block, BlocksSubject};
use crate::{
    infra::record::{PacketBuilder, RecordPacket, ToPacket},
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Block {
    type Opts = MsgPayload;
    fn build_packets(msg_payload: &Self::Opts) -> Vec<RecordPacket> {
        let block = msg_payload.block();
        let block_height = *msg_payload.metadata.block_height;
        let block_producer = (*msg_payload.metadata.block_producer).clone();
        let subject = BlocksSubject {
            producer: Some(block_producer),
            da_height: Some(block.header.da_height.to_owned()),
            height: Some(block_height),
        }
        .dyn_arc();
        let timestamps = msg_payload.timestamp();
        let packet = block.to_packet(&subject, timestamps);
        let packet = match msg_payload.namespace.clone() {
            Some(ns) => packet.with_namespace(&ns),
            _ => packet,
        };
        std::iter::once(packet).collect()
    }
}

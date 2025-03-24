use async_trait::async_trait;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};

use super::{Block, BlocksSubject};
use crate::MsgPayload;

#[async_trait]
impl PacketBuilder for Block {
    type Opts = MsgPayload;
    fn build_packets(msg_payload: &Self::Opts) -> Vec<RecordPacket> {
        let block = msg_payload.block();
        let block_height = *msg_payload.metadata.block_height;
        let block_producer = (*msg_payload.metadata.block_producer).clone();
        let subject = BlocksSubject {
            height: Some(block_height),
            da_height: Some(block.header.da_height.to_owned()),
            producer: Some(block_producer),
            version: Some(block.version.to_owned()),
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

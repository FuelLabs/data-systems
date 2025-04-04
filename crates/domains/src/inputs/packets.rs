use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_types::{BlockTimestamp, TxId};
use rayon::prelude::*;

use super::{subjects::*, Input, InputsQuery};
use crate::{
    blocks::BlockHeight,
    infra::{
        record::{PacketBuilder, RecordPacket, ToPacket},
        RecordPointer,
    },
    transactions::Transaction,
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Input {
    type Opts = (MsgPayload, usize, Transaction);
    fn build_packets(
        (msg_payload, tx_index, tx): &Self::Opts,
    ) -> Vec<RecordPacket> {
        let tx_id = tx.id.clone();
        let block_height = msg_payload.block_height();
        let timestamps = msg_payload.timestamp();
        tx.inputs
            .par_iter()
            .enumerate()
            .map(move |(input_index, input)| {
                let subject = DynInputSubject::new(
                    input,
                    block_height,
                    tx_id.clone(),
                    *tx_index as i32,
                    input_index as i32,
                );
                let pointer = RecordPointer {
                    block_height,
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(*tx_index as u32),
                    input_index: Some(input_index as u32),
                    ..Default::default()
                };
                let packet = subject.build_packet(input, timestamps, pointer);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

pub enum DynInputSubject {
    Contract(InputsContractSubject),
    Coin(InputsCoinSubject),
    Message(InputsMessageSubject),
}

impl DynInputSubject {
    pub fn new(
        input: &Input,
        block_height: BlockHeight,
        tx_id: TxId,
        tx_index: i32,
        input_index: i32,
    ) -> Self {
        match input {
            Input::Contract(contract) => {
                Self::Contract(InputsContractSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    input_index: Some(input_index),
                    contract: Some(contract.contract_id.to_owned().into()),
                })
            }
            Input::Coin(coin) => Self::Coin(InputsCoinSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                input_index: Some(input_index),
                owner: Some(coin.owner.to_owned()),
                asset: Some(coin.asset_id.to_owned()),
            }),
            Input::Message(message) => Self::Message(InputsMessageSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                input_index: Some(input_index),
                sender: Some(message.sender.to_owned()),
                recipient: Some(message.recipient.to_owned()),
            }),
        }
    }

    pub fn build_packet(
        &self,
        input: &Input,
        block_timestamp: BlockTimestamp,
        pointer: RecordPointer,
    ) -> RecordPacket {
        match self {
            Self::Contract(subject) => input.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
            Self::Coin(subject) => input.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
            Self::Message(subject) => input.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
        }
    }

    pub fn to_query_params(&self) -> InputsQuery {
        match self {
            Self::Contract(subject) => InputsQuery::from(subject.to_owned()),
            Self::Coin(subject) => InputsQuery::from(subject.to_owned()),
            Self::Message(subject) => InputsQuery::from(subject.to_owned()),
        }
    }
}

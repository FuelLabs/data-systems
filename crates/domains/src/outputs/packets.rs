use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_types::{BlockTimestamp, ContractId, TxId};
use rayon::prelude::*;

use super::{subjects::*, types::*, OutputsQuery};
use crate::{
    blocks::BlockHeight,
    infra::{
        record::{PacketBuilder, RecordPacket, ToPacket},
        RecordPointer,
    },
    inputs::Input,
    transactions::Transaction,
    MsgPayload,
};

#[async_trait]
impl PacketBuilder for Output {
    type Opts = (MsgPayload, usize, Transaction);
    fn build_packets(
        (msg_payload, tx_index, tx): &Self::Opts,
    ) -> Vec<RecordPacket> {
        let tx_id = tx.id.clone();
        let block_height = msg_payload.block_height();
        tx.outputs
            .par_iter()
            .enumerate()
            .map(|(output_index, output)| {
                let subject = DynOutputSubject::new(
                    output,
                    block_height,
                    tx_id.to_owned(),
                    *tx_index as i32,
                    output_index as i32,
                    tx,
                );
                let timestamps = msg_payload.timestamp();
                let pointer = RecordPointer {
                    block_height,
                    tx_id: Some(tx_id.to_owned()),
                    tx_index: Some(*tx_index as u32),
                    output_index: Some(output_index as u32),
                    ..Default::default()
                };
                let packet = subject.build_packet(output, timestamps, pointer);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

pub enum DynOutputSubject {
    Coin(OutputsCoinSubject),
    Contract(OutputsContractSubject),
    Change(OutputsChangeSubject),
    Variable(OutputsVariableSubject),
    ContractCreated(OutputsContractCreatedSubject),
}

impl DynOutputSubject {
    pub fn new(
        output: &Output,
        block_height: BlockHeight,
        tx_id: TxId,
        tx_index: i32,
        output_index: i32,
        transaction: &Transaction,
    ) -> Self {
        match output {
            Output::Coin(coin) => Self::Coin(OutputsCoinSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to: Some(coin.to.to_owned()),
                asset: Some(coin.asset_id.to_owned()),
            }),
            Output::Contract(contract) => {
                let contract_id =
                    find_output_contract_id(transaction, contract)
                        .unwrap_or_default();
                Self::Contract(OutputsContractSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    output_index: Some(output_index),
                    contract: Some(contract_id),
                })
            }
            Output::Change(change) => Self::Change(OutputsChangeSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to: Some(change.to.to_owned()),
                asset: Some(change.asset_id.to_owned()),
            }),
            Output::Variable(variable) => {
                Self::Variable(OutputsVariableSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    output_index: Some(output_index),
                    to: Some(variable.to.to_owned()),
                    asset: Some(variable.asset_id.to_owned()),
                })
            }
            Output::ContractCreated(created) => {
                Self::ContractCreated(OutputsContractCreatedSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    output_index: Some(output_index),
                    contract: Some(created.contract_id.to_owned()),
                })
            }
        }
    }

    pub fn build_packet(
        &self,
        output: &Output,
        block_timestamp: BlockTimestamp,
        pointer: RecordPointer,
    ) -> RecordPacket {
        match self {
            Self::Coin(subject) => output.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
            Self::Contract(subject) => output.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
            Self::Change(subject) => output.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
            Self::Variable(subject) => output.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
            Self::ContractCreated(subject) => output.to_packet(
                &Arc::new(subject.clone()),
                block_timestamp,
                pointer,
            ),
        }
    }

    pub fn to_query_params(&self) -> OutputsQuery {
        match self {
            Self::Coin(subject) => OutputsQuery::from(subject.to_owned()),
            Self::Contract(subject) => OutputsQuery::from(subject.to_owned()),
            Self::Change(subject) => OutputsQuery::from(subject.to_owned()),
            Self::Variable(subject) => OutputsQuery::from(subject.to_owned()),
            Self::ContractCreated(subject) => {
                OutputsQuery::from(subject.to_owned())
            }
        }
    }
}

pub fn find_output_contract_id(
    tx: &Transaction,
    contract: &OutputContract,
) -> Option<ContractId> {
    let input_index = contract.input_index as usize;
    tx.inputs.get(input_index).and_then(|input| {
        if let Input::Contract(input_contract) = input {
            Some(input_contract.contract_id.to_owned().into())
        } else {
            None
        }
    })
}

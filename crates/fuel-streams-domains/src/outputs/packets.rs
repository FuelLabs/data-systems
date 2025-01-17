use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};
use fuel_streams_types::{ContractId, TxId};
use rayon::prelude::*;

use super::{subjects::*, types::*};
use crate::{
    blocks::BlockHeight,
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
        let block_height = msg_payload.block_height();
        let tx_id = tx.id.clone();
        tx.outputs
            .par_iter()
            .enumerate()
            .map(|(output_index, output)| {
                let subject = main_subject(
                    block_height.clone(),
                    *tx_index as u32,
                    output_index as u32,
                    tx_id.to_owned(),
                    tx,
                    output,
                );
                let packet = output.to_packet(&subject);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

fn main_subject(
    block_height: BlockHeight,
    tx_index: u32,
    output_index: u32,
    tx_id: TxId,
    transaction: &Transaction,
    output: &Output,
) -> Arc<dyn IntoSubject> {
    match output {
        Output::Coin(OutputCoin { to, asset_id, .. }) => OutputsCoinSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            output_index: Some(output_index),
            to: Some(to.to_owned()),
            asset: Some(asset_id.to_owned()),
        }
        .arc(),
        Output::Contract(contract) => {
            let contract_id =
                match find_output_contract_id(transaction, contract) {
                    Some(contract_id) => contract_id,
                    None => {
                        tracing::warn!(
                            "Contract ID not found for output: {:?}",
                            output
                        );

                        Default::default()
                    }
                };

            OutputsContractSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                contract: Some(contract_id),
            }
            .arc()
        }
        Output::Change(OutputChange { to, asset_id, .. }) => {
            OutputsChangeSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }
            .arc()
        }
        Output::Variable(OutputVariable { to, asset_id, .. }) => {
            OutputsVariableSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }
            .arc()
        }
        Output::ContractCreated(OutputContractCreated {
            contract_id, ..
        }) => OutputsContractCreatedSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            output_index: Some(output_index),
            contract: Some(contract_id.to_owned()),
        }
        .arc(),
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

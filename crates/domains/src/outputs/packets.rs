use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};
use fuel_streams_subject::subject::IntoSubject;
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
        let tx_id = tx.id.clone();
        tx.outputs
            .par_iter()
            .enumerate()
            .map(|(output_index, output)| {
                let subject = DynOutputSubject::from((
                    output,
                    msg_payload.block_height(),
                    tx_id.to_owned(),
                    *tx_index as u32,
                    output_index as u32,
                    tx,
                ));
                let packet = output.to_packet(&subject.into());
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

pub struct DynOutputSubject(Arc<dyn IntoSubject>);
impl From<(&Output, BlockHeight, TxId, u32, u32, &Transaction)>
    for DynOutputSubject
{
    fn from(
        (output, block_height, tx_id, tx_index, output_index, transaction): (
            &Output,
            BlockHeight,
            TxId,
            u32,
            u32,
            &Transaction,
        ),
    ) -> Self {
        DynOutputSubject(match output {
            Output::Coin(coin) => OutputsCoinSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to: Some(coin.to.to_owned()),
                asset: Some(coin.asset_id.to_owned()),
            }
            .arc(),
            Output::Contract(contract) => {
                let contract_id =
                    find_output_contract_id(transaction, contract)
                        .unwrap_or_default();
                OutputsContractSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    output_index: Some(output_index),
                    contract: Some(contract_id),
                }
                .arc()
            }
            Output::Change(change) => OutputsChangeSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to: Some(change.to.to_owned()),
                asset: Some(change.asset_id.to_owned()),
            }
            .arc(),
            Output::Variable(variable) => OutputsVariableSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to: Some(variable.to.to_owned()),
                asset: Some(variable.asset_id.to_owned()),
            }
            .arc(),
            Output::ContractCreated(created) => OutputsContractCreatedSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                contract: Some(created.contract_id.to_owned()),
            }
            .arc(),
        })
    }
}

impl From<DynOutputSubject> for Arc<dyn IntoSubject> {
    fn from(subject: DynOutputSubject) -> Self {
        subject.0
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

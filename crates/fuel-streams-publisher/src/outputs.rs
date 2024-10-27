use std::sync::Arc;

use fuel_core_types::fuel_tx::{output::contract::Contract, Output};
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{
    identifiers::{Identifier, IdsExtractable, PacketIdBuilder},
    packets::{PublishError, PublishOpts, PublishPacket},
};

pub fn publish_tasks(
    tx: &Transaction,
    stream: &Stream<Output>,
    opts: &Arc<PublishOpts>,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let tx_id = tx.id(&opts.chain_id);
    let packets: Vec<PublishPacket<Output>> = tx
        .outputs()
        .par_iter()
        .enumerate()
        .flat_map(|(index, output)| {
            let ids = output.extract_ids(Some(tx));
            let mut packets = output.packets_from_ids(ids);
            let packet = packet_from_output(output, tx_id.into(), index, tx);
            packets.push(packet);
            packets
        })
        .collect();

    packets
        .iter()
        .map(|packet| {
            packet.publish(Arc::new(stream.to_owned()), Arc::clone(opts))
        })
        .collect()
}

fn packet_from_output(
    output: &Output,
    tx_id: Bytes32,
    index: usize,
    transaction: &Transaction,
) -> PublishPacket<Output> {
    match output {
        Output::Coin { to, asset_id, .. } => PublishPacket::new(
            output,
            OutputsCoinSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_to(Some((*to).into()))
                .with_asset_id(Some((*asset_id).into()))
                .arc(),
            OutputsCoinSubject::WILDCARD,
        ),
        Output::Contract(contract) => {
            let contract_id = find_output_contract_id(transaction, contract)
                .ok_or_else(|| anyhow::anyhow!("Contract input not found"))
                .unwrap_or_default();

            PublishPacket::new(
                output,
                OutputsContractSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index as u16))
                    .with_contract_id(Some(contract_id.into()))
                    .arc(),
                OutputsContractSubject::WILDCARD,
            )
        }
        Output::Change { to, asset_id, .. } => PublishPacket::new(
            output,
            OutputsChangeSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_to(Some((*to).into()))
                .with_asset_id(Some((*asset_id).into()))
                .arc(),
            OutputsChangeSubject::WILDCARD,
        ),
        Output::Variable { to, asset_id, .. } => PublishPacket::new(
            output,
            OutputsVariableSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_to(Some((*to).into()))
                .with_asset_id(Some((*asset_id).into()))
                .arc(),
            OutputsVariableSubject::WILDCARD,
        ),
        Output::ContractCreated { contract_id, .. } => PublishPacket::new(
            output,
            OutputsContractCreatedSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_contract_id(Some((*contract_id).into()))
                .arc(),
            OutputsContractCreatedSubject::WILDCARD,
        ),
    }
}

pub fn find_output_contract_id(
    tx: &Transaction,
    contract: &Contract,
) -> Option<fuel_core_types::fuel_tx::ContractId> {
    let input_index = contract.input_index as usize;
    tx.inputs().get(input_index).and_then(|input| {
        if let Input::Contract(input_contract) = input {
            Some(input_contract.contract_id)
        } else {
            None
        }
    })
}

impl IdsExtractable for Output {
    fn extract_ids(&self, tx: Option<&Transaction>) -> Vec<Identifier> {
        match self {
            Output::Change { to, asset_id, .. }
            | Output::Variable { to, asset_id, .. }
            | Output::Coin { to, asset_id, .. } => {
                vec![
                    Identifier::Address(to.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
            Output::Contract(contract) => {
                if let Some(tx) = tx {
                    find_output_contract_id(tx, contract)
                        .map(|contract_id| {
                            vec![Identifier::ContractId(contract_id.into())]
                        })
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            }
            Output::ContractCreated { contract_id, .. } => {
                vec![Identifier::ContractId(contract_id.into())]
            }
        }
    }
}

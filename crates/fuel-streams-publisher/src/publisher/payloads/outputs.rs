use std::sync::Arc;

use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{publish, PublishOpts};

pub fn publish_tasks(
    tx: &FuelCoreTransaction,
    tx_id: &Bytes32,
    stream: &Stream<Output>,
    opts: &Arc<PublishOpts>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let packets: Vec<PublishPacket<Output>> = tx
        .outputs()
        .par_iter()
        .enumerate()
        .flat_map(|(index, output)| {
            let main_subject = main_subject(output, tx, tx_id, index);
            let identifier_subjects =
                identifiers(output, tx, tx_id, index as u8)
                    .into_par_iter()
                    .map(|identifier| identifier.into())
                    .map(|subject: OutputsByIdSubject| subject.arc())
                    .collect::<Vec<_>>();

            let output: Output = output.into();

            let mut packets = vec![output.to_packet(main_subject)];
            packets.extend(
                identifier_subjects
                    .into_iter()
                    .map(|subject| output.to_packet(subject)),
            );

            packets
        })
        .collect();

    packets
        .iter()
        .map(|packet| publish(packet, Arc::new(stream.to_owned()), opts))
        .collect()
}

fn main_subject(
    output: &FuelCoreOutput,
    transaction: &FuelCoreTransaction,
    tx_id: &Bytes32,
    index: usize,
) -> Arc<dyn IntoSubject> {
    match output {
        FuelCoreOutput::Coin { to, asset_id, .. } => OutputsCoinSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index as u16),
            to: Some((*to).into()),
            asset_id: Some((*asset_id).into()),
        }
        .arc(),

        FuelCoreOutput::Contract(contract) => {
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
                tx_id: Some(tx_id.to_owned()),
                index: Some(index as u16),
                contract_id: Some(contract_id.into()),
            }
            .arc()
        }

        FuelCoreOutput::Change { to, asset_id, .. } => OutputsChangeSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index as u16),
            to: Some((*to).into()),
            asset_id: Some((*asset_id).into()),
        }
        .arc(),

        FuelCoreOutput::Variable { to, asset_id, .. } => {
            OutputsVariableSubject {
                tx_id: Some(tx_id.to_owned()),
                index: Some(index as u16),
                to: Some((*to).into()),
                asset_id: Some((*asset_id).into()),
            }
            .arc()
        }

        FuelCoreOutput::ContractCreated { contract_id, .. } => {
            OutputsContractCreatedSubject {
                tx_id: Some(tx_id.to_owned()),
                index: Some(index as u16),
                contract_id: Some((*contract_id).into()),
            }
            .arc()
        }
    }
}

pub fn identifiers(
    output: &FuelCoreOutput,
    tx: &FuelCoreTransaction,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    match output {
        FuelCoreOutput::Change { to, asset_id, .. }
        | FuelCoreOutput::Variable { to, asset_id, .. }
        | FuelCoreOutput::Coin { to, asset_id, .. } => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        FuelCoreOutput::Contract(contract) => {
            find_output_contract_id(tx, contract)
                .map(|contract_id| {
                    vec![Identifier::ContractID(
                        tx_id.to_owned(),
                        index,
                        contract_id.into(),
                    )]
                })
                .unwrap_or_default()
        }
        FuelCoreOutput::ContractCreated { contract_id, .. } => {
            vec![Identifier::ContractID(
                tx_id.to_owned(),
                index,
                contract_id.into(),
            )]
        }
    }
}

pub fn find_output_contract_id(
    tx: &FuelCoreTransaction,
    contract: &FuelCoreOutputContract,
) -> Option<fuel_core_types::fuel_tx::ContractId> {
    let input_index = contract.input_index as usize;
    tx.inputs().get(input_index).and_then(|input| {
        if let FuelCoreInput::Contract(input_contract) = input {
            Some(input_contract.contract_id)
        } else {
            None
        }
    })
}

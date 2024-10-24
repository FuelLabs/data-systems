use fuel_core_types::fuel_tx::{output::contract::Contract, Output};
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use rayon::prelude::*;

use crate::{
    identifiers::{Identifier, IdsExtractable, SubjectPayloadBuilder},
    PublishPayload,
    SubjectPayload,
};

pub fn create_publish_payloads(
    stream: &Stream<Output>,
    tx: &Transaction,
    chain_id: &ChainId,
) -> Vec<PublishPayload<Output>> {
    let tx_id = tx.id(chain_id);

    tx.outputs()
        .par_iter()
        .enumerate()
        .map(|(index, output)| {
            let subjects = main_subjects(output, tx_id.into(), index, tx)
                .into_iter()
                .chain(OutputsByIdSubject::build_subjects_payload(
                    tx,
                    &[output.to_owned()],
                ))
                .collect();

            PublishPayload {
                subjects,
                stream: stream.to_owned(),
                payload: output.to_owned(),
            }
        })
        .collect()
}

fn main_subjects(
    output: &Output,
    tx_id: Bytes32,
    index: usize,
    transaction: &Transaction,
) -> Vec<SubjectPayload> {
    match output {
        Output::Coin { to, asset_id, .. } => vec![(
            OutputsCoinSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_to(Some((*to).into()))
                .with_asset_id(Some((*asset_id).into()))
                .boxed(),
            OutputsCoinSubject::WILDCARD,
        )],
        Output::Contract(contract) => {
            let contract_id = find_output_contract_id(transaction, contract)
                .ok_or_else(|| anyhow::anyhow!("Contract input not found"))
                .unwrap_or_default();

            vec![(
                OutputsContractSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index as u16))
                    .with_contract_id(Some(contract_id.into()))
                    .boxed(),
                OutputsContractSubject::WILDCARD,
            )]
        }
        Output::Change { to, asset_id, .. } => vec![(
            OutputsChangeSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_to(Some((*to).into()))
                .with_asset_id(Some((*asset_id).into()))
                .boxed(),
            OutputsChangeSubject::WILDCARD,
        )],
        Output::Variable { to, asset_id, .. } => vec![(
            OutputsVariableSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_to(Some((*to).into()))
                .with_asset_id(Some((*asset_id).into()))
                .boxed(),
            OutputsVariableSubject::WILDCARD,
        )],
        Output::ContractCreated { contract_id, .. } => vec![(
            OutputsContractCreatedSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index as u16))
                .with_contract_id(Some((*contract_id).into()))
                .boxed(),
            OutputsContractCreatedSubject::WILDCARD,
        )],
    }
}

pub fn find_output_contract_id(
    transaction: &Transaction,
    contract: &Contract,
) -> Option<fuel_core_types::fuel_tx::ContractId> {
    let input_index = contract.input_index as usize;
    transaction.inputs().get(input_index).and_then(|input| {
        if let Input::Contract(input_contract) = input {
            Some(input_contract.contract_id)
        } else {
            None
        }
    })
}

impl IdsExtractable for Output {
    fn extract_identifiers(&self, tx: &Transaction) -> Vec<Identifier> {
        match self {
            Output::Change { to, asset_id, .. }
            | Output::Variable { to, asset_id, .. }
            | Output::Coin { to, asset_id, .. } => {
                vec![
                    Identifier::Address(to.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
            Output::Contract(contract) => find_output_contract_id(tx, contract)
                .map(|contract_id| {
                    vec![Identifier::ContractId(contract_id.into())]
                })
                .unwrap_or_default(),
            Output::ContractCreated { contract_id, .. } => {
                vec![Identifier::ContractId(contract_id.into())]
            }
        }
    }
}

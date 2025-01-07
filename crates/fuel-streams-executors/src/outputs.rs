use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Output> {
    pub fn process(
        &self,
        tx: &Transaction,
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let tx_id = tx.id.clone();
        let packets = tx
            .outputs
            .par_iter()
            .enumerate()
            .flat_map(|(index, output)| {
                let main_subject = main_subject(output, tx, &tx_id, index);
                let identifier_subjects =
                    identifiers(output, tx, &tx_id, index as u8)
                        .into_par_iter()
                        .map(|identifier| identifier.into())
                        .map(|subject: OutputsByIdSubject| subject.parse())
                        .collect::<Vec<String>>();

                let mut packets = vec![output.to_packet(main_subject)];
                packets.extend(
                    identifier_subjects
                        .into_iter()
                        .map(|subject| output.to_packet(subject)),
                );

                packets
            })
            .collect::<Vec<_>>();

        packets.iter().map(|packet| self.publish(packet)).collect()
    }
}

fn main_subject(
    output: &Output,
    transaction: &Transaction,
    tx_id: &Bytes32,
    index: usize,
) -> String {
    match output {
        Output::Coin(OutputCoin { to, asset_id, .. }) => OutputsCoinSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index as u16),
            to: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
        }
        .parse(),
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
                tx_id: Some(tx_id.to_owned()),
                index: Some(index as u16),
                contract_id: Some(contract_id),
            }
            .parse()
        }
        Output::Change(OutputChange { to, asset_id, .. }) => {
            OutputsChangeSubject {
                tx_id: Some(tx_id.to_owned()),
                index: Some(index as u16),
                to: Some(to.to_owned()),
                asset_id: Some(asset_id.to_owned()),
            }
            .parse()
        }
        Output::Variable(OutputVariable { to, asset_id, .. }) => {
            OutputsVariableSubject {
                tx_id: Some(tx_id.to_owned()),
                index: Some(index as u16),
                to: Some(to.to_owned()),
                asset_id: Some(asset_id.to_owned()),
            }
            .parse()
        }
        Output::ContractCreated(OutputContractCreated {
            contract_id, ..
        }) => OutputsContractCreatedSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index as u16),
            contract_id: Some(contract_id.to_owned()),
        }
        .parse(),
    }
}

pub fn identifiers(
    output: &Output,
    tx: &Transaction,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    match output {
        Output::Change(OutputChange { to, asset_id, .. })
        | Output::Variable(OutputVariable { to, asset_id, .. })
        | Output::Coin(OutputCoin { to, asset_id, .. }) => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        Output::Contract(contract) => find_output_contract_id(tx, contract)
            .map(|contract_id| {
                vec![Identifier::ContractID(
                    tx_id.to_owned(),
                    index,
                    contract_id.into(),
                )]
            })
            .unwrap_or_default(),
        Output::ContractCreated(OutputContractCreated {
            contract_id, ..
        }) => {
            vec![Identifier::ContractID(
                tx_id.to_owned(),
                index,
                contract_id.into(),
            )]
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

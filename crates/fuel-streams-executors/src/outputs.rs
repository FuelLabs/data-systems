use fuel_streams_core::{subjects::*, types::*};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Output> {
    pub fn process(
        &self,
        (tx_index, tx): (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let block_height = self.block_height();
        let tx_id = tx.id.clone();
        let packets = tx
            .outputs
            .par_iter()
            .enumerate()
            .flat_map(|(output_index, output)| {
                let main_subject = main_subject(
                    block_height.clone(),
                    tx_index as u32,
                    output_index as u32,
                    tx_id.to_owned(),
                    tx,
                    output,
                );
                vec![output.to_packet(main_subject)]
            })
            .collect::<Vec<_>>();

        packets.iter().map(|packet| self.publish(packet)).collect()
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
            to_address: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
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
                contract_id: Some(contract_id),
            }
            .arc()
        }
        Output::Change(OutputChange { to, asset_id, .. }) => {
            OutputsChangeSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to_address: Some(to.to_owned()),
                asset_id: Some(asset_id.to_owned()),
            }
            .arc()
        }
        Output::Variable(OutputVariable { to, asset_id, .. }) => {
            OutputsVariableSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                output_index: Some(output_index),
                to_address: Some(to.to_owned()),
                asset_id: Some(asset_id.to_owned()),
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
            contract_id: Some(contract_id.to_owned()),
        }
        .arc(),
    }
}

// pub fn identifiers(
//     output: &Output,
//     tx: &Transaction,
//     tx_id: &Bytes32,
//     index: u8,
// ) -> Vec<Identifier> {
//     match output {
//         Output::Change(OutputChange { to, asset_id, .. })
//         | Output::Variable(OutputVariable { to, asset_id, .. })
//         | Output::Coin(OutputCoin { to, asset_id, .. }) => {
//             vec![
//                 Identifier::Address(tx_id.to_owned(), index, to.into()),
//                 Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
//             ]
//         }
//         Output::Contract(contract) => find_output_contract_id(tx, contract)
//             .map(|contract_id| {
//                 vec![Identifier::ContractID(
//                     tx_id.to_owned(),
//                     index,
//                     contract_id.into(),
//                 )]
//             })
//             .unwrap_or_default(),
//         Output::ContractCreated(OutputContractCreated {
//             contract_id, ..
//         }) => {
//             vec![Identifier::ContractID(
//                 tx_id.to_owned(),
//                 index,
//                 contract_id.into(),
//             )]
//         }
//     }
// }

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

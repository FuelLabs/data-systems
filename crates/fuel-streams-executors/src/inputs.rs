use fuel_streams_core::{subjects::*, types::*};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Input> {
    pub fn process(
        &self,
        (tx_index, tx): (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let block_height = self.block_height();
        let tx_id = tx.id.clone();
        let packets = tx
            .inputs
            .par_iter()
            .enumerate()
            .flat_map(move |(input_index, input)| {
                let main_subject = main_subject(
                    block_height.clone(),
                    tx_index as u32,
                    input_index as u32,
                    tx_id.clone(),
                    input,
                );
                vec![input.to_packet(main_subject)]
            })
            .collect::<Vec<_>>();

        packets.iter().map(|packet| self.publish(packet)).collect()
    }
}

fn main_subject(
    block_height: BlockHeight,
    tx_index: u32,
    input_index: u32,
    tx_id: TxId,
    input: &Input,
) -> Arc<dyn IntoSubject> {
    match input {
        Input::Contract(contract) => InputsContractSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            input_index: Some(input_index),
            contract_id: Some(contract.contract_id.to_owned().into()),
        }
        .arc(),
        Input::Coin(coin) => InputsCoinSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            input_index: Some(input_index),
            owner: Some(coin.owner.to_owned()),
            asset_id: Some(coin.asset_id.to_owned()),
        }
        .arc(),
        Input::Message(message) => InputsMessageSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            input_index: Some(input_index),
            sender: Some(message.sender.to_owned()),
            recipient: Some(message.recipient.to_owned()),
        }
        .arc(),
    }
}

// pub fn identifiers(
//     input: &Input,
//     tx_id: &Bytes32,
//     index: u8,
// ) -> Vec<Identifier> {
//     let mut identifiers = match input {
//         Input::Coin(coin) => {
//             vec![
//                 Identifier::Address(
//                     tx_id.to_owned(),
//                     index,
//                     coin.owner.to_owned().into(),
//                 ),
//                 Identifier::AssetID(
//                     tx_id.to_owned(),
//                     index,
//                     coin.asset_id.to_owned().into(),
//                 ),
//             ]
//         }
//         Input::Message(message) => {
//             vec![
//                 Identifier::Address(
//                     tx_id.to_owned(),
//                     index,
//                     message.sender.to_owned().into(),
//                 ),
//                 Identifier::Address(
//                     tx_id.to_owned(),
//                     index,
//                     message.recipient.to_owned().into(),
//                 ),
//             ]
//         }
//         Input::Contract(contract) => {
//             vec![Identifier::ContractID(
//                 tx_id.to_owned(),
//                 index,
//                 contract.contract_id.to_owned(),
//             )]
//         }
//     };

//     match input {
//         Input::Coin(InputCoin { predicate, .. })
//         | Input::Message(InputMessage { predicate, .. }) => {
//             let predicate_tag = super::sha256(&predicate.0 .0);
//             identifiers.push(Identifier::PredicateID(
//                 tx_id.to_owned(),
//                 index,
//                 predicate_tag,
//             ));
//         }
//         _ => {}
//     };

//     identifiers
// }

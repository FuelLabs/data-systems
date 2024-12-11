use std::sync::Arc;

use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::prelude::*;

impl Executor<Input> {
    pub fn process(
        &self,
        tx: &Transaction,
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let tx_id = tx.id.clone();
        let packets = tx
            .inputs
            .par_iter()
            .enumerate()
            .flat_map(move |(index, input)| {
                let main_subject = main_subject(input, tx_id.clone(), index);
                let identifier_subjects =
                    identifiers(input, &tx_id, index as u8)
                        .into_par_iter()
                        .map(|identifier| identifier.into())
                        .map(|subject: InputsByIdSubject| subject.arc())
                        .collect::<Vec<_>>();

                let mut packets = vec![input.to_packet(main_subject)];
                packets.extend(
                    identifier_subjects
                        .into_iter()
                        .map(|subject| input.to_packet(subject)),
                );

                packets
            })
            .collect::<Vec<_>>();

        packets.iter().map(|packet| self.publish(packet)).collect()
    }
}

fn main_subject(
    input: &Input,
    tx_id: Bytes32,
    index: usize,
) -> Arc<dyn IntoSubject> {
    match input {
        Input::Contract(contract) => InputsContractSubject {
            tx_id: Some(tx_id),
            index: Some(index),
            contract_id: Some(contract.contract_id.to_owned().into()),
        }
        .arc(),
        Input::Coin(coin) => InputsCoinSubject {
            tx_id: Some(tx_id),
            index: Some(index),
            owner: Some(coin.owner.to_owned()),
            asset_id: Some(coin.asset_id.to_owned()),
        }
        .arc(),
        Input::Message(message) => InputsMessageSubject {
            tx_id: Some(tx_id),
            index: Some(index),
            sender: Some(message.sender.to_owned()),
            recipient: Some(message.recipient.to_owned()),
        }
        .arc(),
    }
}

pub fn identifiers(
    input: &Input,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    let mut identifiers = match input {
        Input::Coin(coin) => {
            vec![
                Identifier::Address(
                    tx_id.to_owned(),
                    index,
                    coin.owner.to_owned().into(),
                ),
                Identifier::AssetID(
                    tx_id.to_owned(),
                    index,
                    coin.asset_id.to_owned().into(),
                ),
            ]
        }
        Input::Message(message) => {
            vec![
                Identifier::Address(
                    tx_id.to_owned(),
                    index,
                    message.sender.to_owned().into(),
                ),
                Identifier::Address(
                    tx_id.to_owned(),
                    index,
                    message.recipient.to_owned().into(),
                ),
            ]
        }
        Input::Contract(contract) => {
            vec![Identifier::ContractID(
                tx_id.to_owned(),
                index,
                contract.contract_id.to_owned(),
            )]
        }
    };

    match input {
        Input::Coin(InputCoin { predicate, .. })
        | Input::Message(InputMessage { predicate, .. }) => {
            let predicate_tag = super::sha256(&predicate.0);
            identifiers.push(Identifier::PredicateID(
                tx_id.to_owned(),
                index,
                predicate_tag,
            ));
        }
        _ => {}
    };

    identifiers
}

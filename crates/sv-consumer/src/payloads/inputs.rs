use std::sync::Arc;

use fuel_core_types::fuel_tx::input::{
    coin::{CoinPredicate, CoinSigned},
    message::{
        MessageCoinPredicate,
        MessageCoinSigned,
        MessageDataPredicate,
        MessageDataSigned,
    },
};
use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::publisher::{publish, PublishOpts};

pub fn publish_tasks(
    tx: &FuelCoreTransaction,
    tx_id: &Bytes32,
    stream: &Stream<Input>,
    opts: &Arc<PublishOpts>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let packets = tx
        .inputs()
        .par_iter()
        .enumerate()
        .flat_map(move |(index, input)| {
            let main_subject = main_subject(input, tx_id.clone(), index);
            let identifier_subjects = identifiers(input, tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: InputsByIdSubject| subject.arc())
                .collect::<Vec<_>>();

            let input: Input = input.into();

            let mut packets = vec![input.to_packet(main_subject)];
            packets.extend(
                identifier_subjects
                    .into_iter()
                    .map(|subject| input.to_packet(subject)),
            );

            packets
        })
        .collect::<Vec<_>>();

    packets
        .iter()
        .map(|packet| publish(packet, Arc::new(stream.to_owned()), opts))
        .collect()
}

fn main_subject(
    input: &FuelCoreInput,
    tx_id: Bytes32,
    index: usize,
) -> Arc<dyn IntoSubject> {
    match input {
        FuelCoreInput::Contract(contract) => {
            let contract_id = contract.contract_id;

            InputsContractSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                contract_id: Some(contract_id.into()),
            }
            .arc()
        }
        FuelCoreInput::CoinSigned(CoinSigned {
            owner, asset_id, ..
        })
        | FuelCoreInput::CoinPredicate(CoinPredicate {
            owner, asset_id, ..
        }) => InputsCoinSubject {
            tx_id: Some(tx_id),
            index: Some(index),
            owner: Some(owner.into()),
            asset_id: Some(asset_id.into()),
        }
        .arc(),
        FuelCoreInput::MessageCoinSigned(MessageCoinSigned {
            sender,
            recipient,
            ..
        })
        | FuelCoreInput::MessageCoinPredicate(MessageCoinPredicate {
            sender,
            recipient,
            ..
        })
        | FuelCoreInput::MessageDataSigned(MessageDataSigned {
            sender,
            recipient,
            ..
        })
        | FuelCoreInput::MessageDataPredicate(MessageDataPredicate {
            sender,
            recipient,
            ..
        }) => InputsMessageSubject {
            tx_id: Some(tx_id),
            index: Some(index),
            sender: Some(sender.into()),
            recipient: Some(recipient.into()),
        }
        .arc(),
    }
}

pub fn identifiers(
    input: &FuelCoreInput,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    let mut identifiers = match input {
        FuelCoreInput::CoinSigned(CoinSigned {
            owner, asset_id, ..
        }) => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, owner.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        FuelCoreInput::CoinPredicate(CoinPredicate {
            owner, asset_id, ..
        }) => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, owner.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        FuelCoreInput::MessageCoinSigned(MessageCoinSigned {
            sender,
            recipient,
            ..
        })
        | FuelCoreInput::MessageCoinPredicate(MessageCoinPredicate {
            sender,
            recipient,
            ..
        })
        | FuelCoreInput::MessageDataSigned(MessageDataSigned {
            sender,
            recipient,
            ..
        })
        | FuelCoreInput::MessageDataPredicate(MessageDataPredicate {
            sender,
            recipient,
            ..
        }) => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, sender.into()),
                Identifier::Address(tx_id.to_owned(), index, recipient.into()),
            ]
        }
        FuelCoreInput::Contract(contract) => {
            vec![Identifier::ContractID(
                tx_id.to_owned(),
                index,
                contract.contract_id.into(),
            )]
        }
    };

    if let Some((predicate_bytecode, _, _)) = input.predicate() {
        let predicate_tag = super::sha256(predicate_bytecode);
        identifiers.push(Identifier::PredicateID(
            tx_id.to_owned(),
            index,
            predicate_tag,
        ));
    }

    identifiers
}

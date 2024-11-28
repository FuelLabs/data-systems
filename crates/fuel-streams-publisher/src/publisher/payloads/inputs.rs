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
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use super::identifiers::{Identifier, IdsExtractable, PacketIdBuilder};
use crate::packets::{PublishOpts, PublishPacket};

pub fn publish_tasks(
    tx: &Transaction,
    tx_id: &Bytes32,
    stream: &Stream<Input>,
    opts: &Arc<PublishOpts>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let packets: Vec<PublishPacket<Input>> = tx
        .inputs()
        .par_iter()
        .enumerate()
        .flat_map(move |(index, input)| {
            let ids = input.extract_ids(tx, tx_id, index as u8);
            let mut packets = input.packets_from_ids(ids);
            let packet = packet_from_input(input, tx_id.to_owned(), index);
            packets.push(packet);
            packets
        })
        .collect();

    packets
        .iter()
        .map(|packet| {
            let stream = stream.clone();
            packet.publish(Arc::new(stream.to_owned()), opts)
        })
        .collect()
}

fn packet_from_input(
    input: &Input,
    tx_id: Bytes32,
    index: usize,
) -> PublishPacket<Input> {
    match input {
        Input::Contract(contract) => {
            let contract_id = contract.contract_id;
            PublishPacket::new(
                input,
                InputsContractSubject {
                    tx_id: Some(tx_id),
                    index: Some(index),
                    contract_id: Some(contract_id.into()),
                }
                .arc(),
            )
        }
        Input::CoinSigned(CoinSigned {
            owner, asset_id, ..
        })
        | Input::CoinPredicate(CoinPredicate {
            owner, asset_id, ..
        }) => PublishPacket::new(
            input,
            InputsCoinSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                owner: Some(owner.into()),
                asset_id: Some(asset_id.into()),
            }
            .arc(),
        ),
        Input::MessageCoinSigned(MessageCoinSigned {
            sender,
            recipient,
            ..
        })
        | Input::MessageCoinPredicate(MessageCoinPredicate {
            sender,
            recipient,
            ..
        })
        | Input::MessageDataSigned(MessageDataSigned {
            sender,
            recipient,
            ..
        })
        | Input::MessageDataPredicate(MessageDataPredicate {
            sender,
            recipient,
            ..
        }) => PublishPacket::new(
            input,
            InputsMessageSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                sender: Some(sender.into()),
                recipient: Some(recipient.into()),
            }
            .arc(),
        ),
    }
}

impl IdsExtractable for Input {
    fn extract_ids(
        &self,
        _tx: &Transaction,
        tx_id: &Bytes32,
        index: u8,
    ) -> Vec<Identifier> {
        let mut ids = match self {
            Input::CoinSigned(CoinSigned {
                owner, asset_id, ..
            }) => {
                vec![
                    Identifier::Address(tx_id.to_owned(), index, owner.into()),
                    Identifier::AssetID(
                        tx_id.to_owned(),
                        index,
                        asset_id.into(),
                    ),
                ]
            }
            Input::CoinPredicate(CoinPredicate {
                owner, asset_id, ..
            }) => vec![
                Identifier::Address(tx_id.to_owned(), index, owner.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ],
            Input::MessageCoinSigned(MessageCoinSigned {
                sender,
                recipient,
                ..
            })
            | Input::MessageCoinPredicate(MessageCoinPredicate {
                sender,
                recipient,
                ..
            })
            | Input::MessageDataSigned(MessageDataSigned {
                sender,
                recipient,
                ..
            })
            | Input::MessageDataPredicate(MessageDataPredicate {
                sender,
                recipient,
                ..
            }) => vec![
                Identifier::Address(tx_id.to_owned(), index, sender.into()),
                Identifier::Address(tx_id.to_owned(), index, recipient.into()),
            ],
            Input::Contract(contract) => {
                vec![Identifier::ContractID(
                    tx_id.to_owned(),
                    index,
                    contract.contract_id.into(),
                )]
            }
        };

        if let Some((predicate_bytecode, _, _)) = self.predicate() {
            let predicate_tag = super::sha256(predicate_bytecode);
            ids.push(Identifier::PredicateID(
                tx_id.to_owned(),
                index,
                predicate_tag,
            ))
        }

        ids
    }
}

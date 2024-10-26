use std::sync::Arc;

use fuel_core_types::fuel_tx::{
    input::{
        coin::{CoinPredicate, CoinSigned},
        message::{
            MessageCoinPredicate,
            MessageCoinSigned,
            MessageDataPredicate,
            MessageDataSigned,
        },
    },
    UniqueIdentifier,
};
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{
    identifiers::{Identifier, IdsExtractable, PacketIdBuilder},
    packets::{PublishError, PublishOpts, PublishPacket},
};

pub fn publish_tasks(
    tx: &Transaction,
    stream: &Stream<Input>,
    opts: &Arc<PublishOpts>,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let tx_id = tx.id(&opts.chain_id);
    tx.inputs()
        .par_iter()
        .enumerate()
        .flat_map(move |(index, input)| {
            let ids = input.extract_ids(Some(tx));
            let mut packets = input.packets_from_ids(ids);
            let packet = packet_from_input(input, tx_id.into(), index);
            packets.push(packet);
            packets
        })
        .map(|packet| {
            let stream = stream.clone();
            let opts = Arc::clone(opts);
            packet.publish(Arc::new(stream.to_owned()), Arc::clone(&opts))
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
                InputsContractSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_contract_id(Some(contract_id.into()))
                    .arc(),
                InputsContractSubject::WILDCARD,
            )
        }
        Input::CoinSigned(CoinSigned {
            owner, asset_id, ..
        })
        | Input::CoinPredicate(CoinPredicate {
            owner, asset_id, ..
        }) => PublishPacket::new(
            input,
            InputsCoinSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_owner(Some(owner.into()))
                .with_asset_id(Some(asset_id.into()))
                .arc(),
            InputsCoinSubject::WILDCARD,
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
            InputsMessageSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_sender(Some(sender.into()))
                .with_recipient(Some(recipient.into()))
                .arc(),
            InputsMessageSubject::WILDCARD,
        ),
    }
}

impl IdsExtractable for Input {
    fn extract_ids(&self, _tx: Option<&Transaction>) -> Vec<Identifier> {
        let mut ids = match self {
            Input::CoinSigned(CoinSigned {
                owner, asset_id, ..
            }) => {
                vec![
                    Identifier::Address(owner.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
            Input::CoinPredicate(CoinPredicate {
                owner, asset_id, ..
            }) => {
                vec![
                    Identifier::Address(owner.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
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
            }) => {
                vec![
                    Identifier::Address(sender.into()),
                    Identifier::Address(recipient.into()),
                ]
            }
            Input::Contract(contract) => {
                vec![Identifier::ContractId(contract.contract_id.into())]
            }
        };

        if let Some((predicate_bytecode, _, _)) = self.predicate() {
            let predicate_tag = crate::sha256(predicate_bytecode);
            ids.push(Identifier::PredicateId(predicate_tag))
        }

        ids
    }
}

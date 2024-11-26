use std::sync::Arc;

use fuel_core_types::fuel_tx::{
    input::{
        coin::{CoinPredicate, CoinSigned},
        contract::Contract,
        message::{
            compute_message_id,
            MessageCoinPredicate,
            MessageCoinSigned,
            MessageDataPredicate,
            MessageDataSigned,
        },
    },
    UtxoId,
};
use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{publish, PublishOpts};

pub fn publish_tasks(
    tx: &FuelCoreTransaction,
    tx_id: &Bytes32,
    stream: &Stream<Utxo>,
    opts: &Arc<PublishOpts>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let packets = tx
        .inputs()
        .par_iter()
        .filter_map(|input| utxo_packet(input, tx_id, input.utxo_id().cloned()))
        .collect::<Vec<_>>();

    packets
        .into_iter()
        .map(|packet| publish(&packet, Arc::new(stream.to_owned()), opts))
        .collect()
}

fn utxo_packet(
    input: &FuelCoreInput,
    tx_id: &Bytes32,
    utxo_id: Option<UtxoId>,
) -> Option<PublishPacket<Utxo>> {
    utxo_id?;
    let utxo_id = utxo_id.expect("safe to unwrap utxo");

    match input {
        FuelCoreInput::Contract(Contract { utxo_id, .. }) => {
            let utxo = Utxo {
                utxo_id: utxo_id.to_owned(),
                tx_id: tx_id.to_owned(),
                ..Default::default()
            };

            let subject = UtxosSubject {
                utxo_type: Some(UtxoType::Contract),
                hash: Some(tx_id.to_owned().into()),
            }
            .arc();

            Some(utxo.to_packet(subject))
        }
        FuelCoreInput::CoinSigned(CoinSigned {
            utxo_id, amount, ..
        })
        | FuelCoreInput::CoinPredicate(CoinPredicate {
            utxo_id, amount, ..
        }) => {
            let utxo = Utxo {
                utxo_id: utxo_id.to_owned(),
                amount: Some(*amount),
                tx_id: tx_id.to_owned(),
                ..Default::default()
            };

            let subject = UtxosSubject {
                utxo_type: Some(UtxoType::Coin),
                hash: Some(tx_id.to_owned().into()),
            }
            .arc();

            Some(utxo.to_packet(subject))
        }
        message @ (FuelCoreInput::MessageCoinSigned(MessageCoinSigned {
            amount,
            nonce,
            recipient,
            sender,
            ..
        })
        | FuelCoreInput::MessageCoinPredicate(
            MessageCoinPredicate {
                amount,
                nonce,
                recipient,
                sender,
                ..
            },
        )
        | FuelCoreInput::MessageDataSigned(MessageDataSigned {
            amount,
            nonce,
            recipient,
            sender,
            ..
        })
        | FuelCoreInput::MessageDataPredicate(
            MessageDataPredicate {
                amount,
                nonce,
                recipient,
                sender,
                ..
            },
        )) => {
            let (data, hash) = if let Some(data) = message.input_data() {
                let hash: MessageId =
                    compute_message_id(sender, recipient, nonce, *amount, data)
                        .into();
                (Some(data.to_vec()), hash)
            } else {
                (None, tx_id.to_owned().into())
            };

            let utxo = Utxo {
                utxo_id,
                sender: Some(sender.into()),
                recipient: Some(recipient.into()),
                nonce: Some(nonce.into()),
                amount: Some(*amount),
                tx_id: tx_id.to_owned(),
                data,
            };
            let subject = UtxosSubject {
                utxo_type: Some(UtxoType::Message),
                hash: Some(hash),
            }
            .arc();

            Some(utxo.to_packet(subject))
        }
    }
}

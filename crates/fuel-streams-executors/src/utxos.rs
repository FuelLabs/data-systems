use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Utxo> {
    pub fn process(
        &self,
        tx: &Transaction,
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let tx_id = tx.id.clone();
        let packets = tx
            .inputs
            .par_iter()
            .filter_map(|input| utxo_packet(input, &tx_id))
            .collect::<Vec<_>>();

        packets
            .into_iter()
            .map(|packet| self.publish(&packet))
            .collect()
    }
}

fn utxo_packet(input: &Input, tx_id: &Bytes32) -> Option<PublishPacket<Utxo>> {
    match input {
        Input::Contract(InputContract { utxo_id, .. }) => {
            let utxo = Utxo {
                utxo_id: utxo_id.to_owned(),
                tx_id: tx_id.to_owned().into(),
                ..Default::default()
            };
            let subject = UtxosSubject {
                utxo_type: Some(UtxoType::Contract),
                utxo_id: Some(utxo_id.into()),
            }
            .arc();
            Some(utxo.to_packet(subject))
        }
        Input::Coin(InputCoin {
            utxo_id, amount, ..
        }) => {
            let utxo = Utxo {
                utxo_id: utxo_id.to_owned(),
                amount: Some(*amount),
                tx_id: tx_id.to_owned().into(),
                ..Default::default()
            };
            let subject = UtxosSubject {
                utxo_type: Some(UtxoType::Coin),
                utxo_id: Some(utxo_id.into()),
            }
            .arc();
            Some(utxo.to_packet(subject))
        }
        Input::Message(
            input @ InputMessage {
                amount,
                nonce,
                recipient,
                sender,
                data,
                ..
            },
        ) => {
            let utxo_id = input.computed_utxo_id();
            let utxo = Utxo {
                tx_id: tx_id.to_owned().into(),
                utxo_id: utxo_id.to_owned(),
                sender: Some(sender.to_owned()),
                recipient: Some(recipient.to_owned()),
                nonce: Some(nonce.to_owned()),
                amount: Some(*amount),
                data: Some(data.to_owned()),
            };
            let subject = UtxosSubject {
                utxo_type: Some(UtxoType::Message),
                utxo_id: None,
            }
            .arc();
            Some(utxo.to_packet(subject))
        }
    }
}

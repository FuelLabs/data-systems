use fuel_streams_core::prelude::*;
use fuel_streams_store::store::StorePacket;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Utxo> {
    pub fn process(
        &self,
        (tx_index, tx): (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let tx_id = tx.id.clone();
        let packets = tx
            .inputs
            .par_iter()
            .enumerate()
            .filter_map(|(index, input)| {
                let order = self
                    .record_order()
                    .with_tx(tx_index as u32)
                    .with_record(index as u32);
                utxo_packet(input, &tx_id, &order)
            })
            .collect::<Vec<_>>();

        packets
            .into_iter()
            .map(|packet| self.publish(&packet))
            .collect()
    }
}

fn utxo_packet(
    input: &Input,
    tx_id: &Bytes32,
    order: &RecordOrder,
) -> Option<StorePacket<Utxo>> {
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
            .parse();
            Some(utxo.to_packet(subject, order))
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
            .parse();
            Some(utxo.to_packet(subject, order))
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
                utxo_id: Some(utxo_id.into()),
            }
            .parse();
            Some(utxo.to_packet(subject, order))
        }
    }
}

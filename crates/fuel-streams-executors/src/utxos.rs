use fuel_streams_core::{subjects::*, types::*};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Utxo> {
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
            .map(|(input_index, input)| {
                let (utxo, subject) = main_subject(
                    block_height.clone(),
                    tx_index as u32,
                    input_index as u32,
                    tx_id.clone(),
                    input,
                );
                utxo.to_packet(subject)
            })
            .collect::<Vec<_>>();

        packets
            .into_iter()
            .map(|packet| self.publish(&packet))
            .collect()
    }
}

fn main_subject(
    block_height: BlockHeight,
    tx_index: u32,
    input_index: u32,
    tx_id: TxId,
    input: &Input,
) -> (Utxo, Arc<dyn IntoSubject>) {
    match input {
        Input::Contract(InputContract { utxo_id, .. }) => {
            let utxo = Utxo {
                utxo_id: utxo_id.to_owned(),
                tx_id: tx_id.to_owned().into(),
                ..Default::default()
            };
            let subject = UtxosSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                input_index: Some(input_index),
                utxo_type: Some(UtxoType::Contract),
                utxo_id: Some(utxo_id.into()),
            }
            .arc();
            (utxo, subject)
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
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                input_index: Some(input_index),
                utxo_type: Some(UtxoType::Coin),
                utxo_id: Some(utxo_id.into()),
            }
            .arc();
            (utxo, subject)
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
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                input_index: Some(input_index),
                utxo_type: Some(UtxoType::Message),
                utxo_id: Some(utxo_id.into()),
            }
            .arc();
            (utxo, subject)
        }
    }
}

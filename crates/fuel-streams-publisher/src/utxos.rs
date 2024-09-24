use std::sync::Arc;

// use fuel_core_storage::transactional::AtomicView;
use fuel_streams_core::{
    prelude::*,
    types::{Transaction, UniqueIdentifier},
    utxos::{
        Utxo,
        UtxosCoinSubject,
        UtxosContractSubject,
        UtxosMessageSubject,
    },
    Stream,
};
use fuel_tx::{input::AsField, MessageId};

use crate::{
    inputs::inputs_from_transaction,
    metrics::PublisherMetrics,
    publish_with_metrics,
    FuelCoreLike,
};

#[derive(Debug, Clone)]
pub struct UtxoMessageData {
    pub data: Option<Vec<u8>>,
    pub hash: MessageId,
}

#[derive(Debug, Clone)]
pub struct UtxoCoinData {
    pub tx_id: Bytes32,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct UtxoContractData {
    pub tx_id: Bytes32,
}

#[derive(Debug, Clone)]
pub enum UtxoType {
    Contract(UtxoContractData),
    Coin(UtxoCoinData),
    Message(UtxoMessageData),
}

#[derive(Debug, Clone)]
enum UtxosSubject {
    Contract(UtxosContractSubject, Utxo),
    Coin(UtxosCoinSubject, Utxo),
    Message(UtxosMessageSubject, Utxo),
}

fn get_utxo_data(input: &Input, tx_id: Bytes32) -> UtxoType {
    match input {
        Input::Contract(_) => UtxoType::Contract(UtxoContractData { tx_id }),
        Input::CoinSigned(c) => UtxoType::Coin(UtxoCoinData {
            tx_id,
            data: c.predicate_data.as_field().cloned(),
        }),
        Input::CoinPredicate(c) => UtxoType::Coin(UtxoCoinData {
            tx_id,
            data: c.predicate_data.as_field().cloned(),
        }),
        Input::MessageCoinSigned(message) => {
            UtxoType::Message(UtxoMessageData {
                hash: message.message_id(),
                data: message.data.as_field().cloned(),
            })
        }
        Input::MessageCoinPredicate(message) => {
            UtxoType::Message(UtxoMessageData {
                hash: message.message_id(),
                data: message.data.as_field().cloned(),
            })
        }
        Input::MessageDataSigned(message) => {
            UtxoType::Message(UtxoMessageData {
                hash: message.message_id(),
                data: message.data.as_field().cloned(),
            })
        }
        Input::MessageDataPredicate(message) => {
            UtxoType::Message(UtxoMessageData {
                hash: message.message_id(),
                data: message.data.as_field().cloned(),
            })
        }
    }
}

pub async fn publish(
    metrics: &Arc<PublisherMetrics>,
    stream: &Stream<Utxo>,
    fuel_core: &dyn FuelCoreLike,
    transactions: &[Transaction],
    block_producer: &fuel_streams_core::types::Address,
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();
    // let off_chain_database = fuel_core.database().off_chain().latest_view()?;
    // off_chain_database.coin(utxo_id).unwrap().
    // off_chain_database.contract_latest_utxo(contract_id)
    // off_chain_database.

    let subjects: Vec<UtxosSubject> = transactions
        .iter()
        .flat_map(|transaction| {
            let tx_id = transaction.id(fuel_core.chain_id());
            let inputs = inputs_from_transaction(transaction);

            inputs
                .iter()
                .map(|input| {
                    let utxo = get_utxo_data(input, tx_id.into());
                    match utxo {
                        UtxoType::Coin(coin) => UtxosSubject::Coin(
                            UtxosCoinSubject::new()
                                .with_tx_id(Some(coin.tx_id)),
                            Utxo(coin.data),
                        ),
                        UtxoType::Contract(contract) => UtxosSubject::Contract(
                            UtxosContractSubject::new()
                                .with_tx_id(Some(contract.tx_id)),
                            Utxo(None),
                        ),
                        UtxoType::Message(message) => UtxosSubject::Message(
                            UtxosMessageSubject::new()
                                .with_hash(Some(message.hash.into())),
                            Utxo(message.data),
                        ),
                    }
                })
                .collect::<Vec<UtxosSubject>>()
        })
        .collect();

    for subject_item in subjects {
        match subject_item {
            UtxosSubject::Contract(subject, payload) => {
                publish_with_metrics!(
                    stream.publish(&subject, &payload),
                    metrics,
                    chain_id,
                    block_producer,
                    UtxosContractSubject::WILDCARD
                );
            }
            UtxosSubject::Coin(subject, payload) => {
                publish_with_metrics!(
                    stream.publish(&subject, &payload),
                    metrics,
                    chain_id,
                    block_producer,
                    UtxosCoinSubject::WILDCARD
                );
            }
            UtxosSubject::Message(subject, payload) => {
                publish_with_metrics!(
                    stream.publish(&subject, &payload),
                    metrics,
                    chain_id,
                    block_producer,
                    UtxosMessageSubject::WILDCARD
                );
            }
        };
    }

    Ok(())
}

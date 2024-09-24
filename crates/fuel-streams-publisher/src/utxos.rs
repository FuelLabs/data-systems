use std::sync::Arc;

use fuel_core_storage::transactional::AtomicView;
use fuel_streams_core::{
    prelude::*,
    types::{Transaction, UniqueIdentifier},
    utxos::{types::UtxoType, Utxo, UtxosSubject},
    Stream,
};
use fuel_tx::{input::AsField, UtxoId};

use crate::{
    inputs::inputs_from_transaction,
    metrics::PublisherMetrics,
    publish_with_metrics,
    FuelCoreLike,
};

fn get_utxo_data(
    input: &Input,
    tx_id: Bytes32,
    utxo_id: Option<UtxoId>,
    fuel_core: &dyn FuelCoreLike,
) -> Option<UtxosSubject> {
    if utxo_id.is_none() {
        return None;
    }
    let utxo_id = utxo_id.expect("safe to unwrap utxo");
    let on_chain_database = fuel_core
        .database()
        .on_chain()
        .latest_view()
        .expect("error getting latest view");

    match input {
        Input::Contract(c) => {
            if !on_chain_database
                .contract_latest_utxo(c.contract_id)
                .ok()
                .is_some()
            {
                return None;
            };
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Contract))
                .with_tx_id(Some(tx_id));
            Some(subject)
        }
        Input::CoinSigned(_) => {
            if !on_chain_database.coin(&utxo_id).ok().is_some() {
                return None;
            };
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Coin))
                .with_tx_id(Some(tx_id));
            Some(subject)
        }
        Input::CoinPredicate(_) => {
            if !on_chain_database.coin(&utxo_id).ok().is_some() {
                return None;
            }
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Coin))
                .with_tx_id(Some(tx_id));
            Some(subject)
        }
        Input::MessageCoinSigned(message) => {
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_tx_id(Some(tx_id))
                .with_hexified_data(message.data.as_field().cloned())
                .with_amount(Some(message.amount.into()))
                .with_nonce(Some(message.nonce.into()))
                .with_recipient(Some(message.recipient.into()))
                .with_sender(Some(message.sender.into()))
                .with_computed_hash();
            Some(subject)
        }
        Input::MessageCoinPredicate(message) => {
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_tx_id(Some(tx_id))
                .with_hexified_data(message.data.as_field().cloned())
                .with_amount(Some(message.amount.into()))
                .with_nonce(Some(message.nonce.into()))
                .with_recipient(Some(message.recipient.into()))
                .with_sender(Some(message.sender.into()))
                .with_computed_hash();
            Some(subject)
        }
        Input::MessageDataSigned(message) => {
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_tx_id(Some(tx_id))
                .with_hexified_data(message.data.as_field().cloned())
                .with_amount(Some(message.amount.into()))
                .with_nonce(Some(message.nonce.into()))
                .with_recipient(Some(message.recipient.into()))
                .with_sender(Some(message.sender.into()))
                .with_computed_hash();
            Some(subject)
        }
        Input::MessageDataPredicate(message) => {
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_tx_id(Some(tx_id))
                .with_hexified_data(message.data.as_field().cloned())
                .with_amount(Some(message.amount.into()))
                .with_nonce(Some(message.nonce.into()))
                .with_recipient(Some(message.recipient.into()))
                .with_sender(Some(message.sender.into()))
                .with_computed_hash();
            Some(subject)
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
    let subjects: Vec<UtxosSubject> = transactions
        .iter()
        .flat_map(|transaction| {
            let tx_id = transaction.id(fuel_core.chain_id());
            let inputs = inputs_from_transaction(transaction);

            inputs
                .iter()
                .filter_map(|input| {
                    let utxo_id = input.utxo_id().cloned();
                    get_utxo_data(input, tx_id.into(), utxo_id, fuel_core)
                })
                .collect::<Vec<UtxosSubject>>()
        })
        .collect();

    let empty_data = Utxo(Some(vec![]));
    for subject in subjects {
        match subject.utxo_type.clone().unwrap_or_default() {
            UtxoType::Contract => {
                publish_with_metrics!(
                    stream.publish(&subject, &empty_data),
                    metrics,
                    chain_id,
                    block_producer,
                    UtxosSubject::WILDCARD
                );
            }
            UtxoType::Coin => {
                publish_with_metrics!(
                    stream.publish(&subject, &empty_data),
                    metrics,
                    chain_id,
                    block_producer,
                    UtxosSubject::WILDCARD
                );
            }
            UtxoType::Message => {
                publish_with_metrics!(
                    stream.publish(&subject, &empty_data),
                    metrics,
                    chain_id,
                    block_producer,
                    UtxosSubject::WILDCARD
                );
            }
        };
    }

    Ok(())
}

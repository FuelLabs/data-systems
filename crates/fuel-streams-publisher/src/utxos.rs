use std::sync::Arc;

use fuel_core_storage::transactional::AtomicView;
use fuel_streams_core::{
    prelude::*,
    transactions::TransactionExt,
    types::Transaction,
    utxos::{
        types::{Utxo, UtxoType},
        UtxosSubject,
    },
    Stream,
};
use fuel_tx::{input::AsField, UtxoId};

use crate::{
    metrics::PublisherMetrics,
    prefix_subject,
    publish_with_metrics,
    FuelCoreLike,
};

fn get_utxo_data(
    input: &Input,
    tx_id: Bytes32,
    utxo_id: Option<UtxoId>,
    fuel_core: &dyn FuelCoreLike,
) -> Option<(UtxosSubject, Utxo)> {
    utxo_id?;
    let utxo_id = utxo_id.expect("safe to unwrap utxo");
    let on_chain_database = fuel_core
        .database()
        .on_chain()
        .latest_view()
        .expect("error getting latest view");
    match input {
        Input::Contract(c) => {
            on_chain_database.contract_latest_utxo(c.contract_id).ok()?;
            let utxo_payload = Utxo::new(
                utxo_id,
                None,
                None,
                None,
                None,
                None,
                tx_id.into_inner(),
            );
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Contract))
                .with_hash(Some(utxo_payload.compute_hash().into()));
            Some((subject, utxo_payload))
        }
        Input::CoinSigned(c) => {
            on_chain_database.coin(&utxo_id).ok()?;
            let utxo_payload = Utxo::new(
                utxo_id,
                None,
                None,
                None,
                None,
                Some(c.amount),
                tx_id.into_inner(),
            );
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Coin))
                .with_hash(Some(utxo_payload.compute_hash().into()));
            Some((subject, utxo_payload))
        }
        Input::CoinPredicate(c) => {
            on_chain_database.coin(&utxo_id).ok()?;
            let utxo_payload = Utxo::new(
                utxo_id,
                None,
                None,
                None,
                None,
                Some(c.amount),
                tx_id.into_inner(),
            );
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Coin))
                .with_hash(Some(utxo_payload.compute_hash().into()));
            Some((subject, utxo_payload))
        }
        Input::MessageCoinSigned(message) => {
            let utxo_payload = Utxo::new(
                utxo_id,
                Some(message.sender),
                Some(message.recipient),
                Some(message.nonce),
                message.data.as_field().cloned(),
                Some(message.amount),
                tx_id.into_inner(),
            );
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_hash(Some(utxo_payload.compute_hash().into()));
            Some((subject, utxo_payload))
        }
        Input::MessageCoinPredicate(message) => {
            let utxo_payload = Utxo::new(
                utxo_id,
                Some(message.sender),
                Some(message.recipient),
                Some(message.nonce),
                message.data.as_field().cloned(),
                Some(message.amount),
                tx_id.into_inner(),
            );
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_hash(Some(utxo_payload.compute_hash().into()));
            Some((subject, utxo_payload))
        }
        Input::MessageDataSigned(message) => {
            let utxo_payload = Utxo::new(
                utxo_id,
                Some(message.sender),
                Some(message.recipient),
                Some(message.nonce),
                message.data.as_field().cloned(),
                Some(message.amount),
                tx_id.into_inner(),
            );
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_hash(Some(utxo_payload.compute_hash().into()));
            Some((subject, utxo_payload))
        }
        Input::MessageDataPredicate(message) => {
            let utxo_payload = Utxo::new(
                utxo_id,
                Some(message.sender),
                Some(message.recipient),
                Some(message.nonce),
                message.data.as_field().cloned(),
                Some(message.amount),
                tx_id.into_inner(),
            );
            let subject = UtxosSubject::new()
                .with_utxo_type(Some(UtxoType::Message))
                .with_hash(Some(utxo_payload.compute_hash().into()));
            Some((subject, utxo_payload))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    metrics: &Arc<PublisherMetrics>,
    stream: &Stream<Utxo>,
    fuel_core: &dyn FuelCoreLike,
    transaction: &Transaction,
    tx_id: Bytes32,
    chain_id: &ChainId,
    block_producer: &fuel_streams_core::types::Address,
    subject_prefix: Option<String>,
) -> anyhow::Result<()> {
    let subjects = transaction
        .inputs()
        .iter()
        .filter_map(|input| {
            let utxo_id = input.utxo_id().cloned();
            get_utxo_data(input, tx_id.clone(), utxo_id, fuel_core)
        })
        .collect::<Vec<(UtxosSubject, Utxo)>>();

    for (subject, utxo) in subjects {
        publish_with_metrics!(
            stream
                .publish_raw(&prefix_subject(&subject_prefix, &subject), &utxo),
            metrics,
            chain_id,
            block_producer,
            UtxosSubject::WILDCARD
        );
    }

    Ok(())
}

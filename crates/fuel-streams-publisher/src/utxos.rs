use std::sync::Arc;

use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::fuel_tx::{input::AsField, UtxoId};
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use futures::{StreamExt, TryStreamExt};
use rayon::prelude::*;

use crate::{
    metrics::PublisherMetrics,
    FuelCoreLike,
    PublishError,
    PublishPayload,
    CONCURRENCY_LIMIT,
};

pub async fn publish_tasks(
    stream: &Stream<Utxo>,
    transactions: &[Transaction],
    chain_id: &ChainId,
    block_producer: &Address,
    metrics: &Arc<PublisherMetrics>,
    fuel_core: &dyn FuelCoreLike,
) -> Result<(), PublishError> {
    futures::stream::iter(
        transactions
            .iter()
            .flat_map(|tx| create_publish_payloads(tx, chain_id, fuel_core)),
    )
    .map(Ok)
    .try_for_each_concurrent(*CONCURRENCY_LIMIT, |payload| {
        let metrics = metrics.clone();
        let chain_id = chain_id.to_owned();
        let block_producer = block_producer.clone();
        async move {
            payload
                .publish(stream, &metrics, &chain_id, &block_producer)
                .await
        }
    })
    .await
}

fn create_publish_payloads(
    tx: &Transaction,
    chain_id: &ChainId,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<PublishPayload<Utxo>> {
    let tx_id = tx.id(chain_id);
    tx.inputs()
        .par_iter()
        .filter_map(|input| {
            find_utxo(input, tx_id.into(), input.utxo_id().cloned(), fuel_core)
        })
        .map(|(subject, utxo)| PublishPayload {
            subject: (subject.boxed(), UtxosSubject::WILDCARD),
            payload: utxo.to_owned(),
        })
        .collect::<Vec<PublishPayload<Utxo>>>()
}

fn find_utxo(
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

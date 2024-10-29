use std::sync::Arc;

use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::fuel_tx::{input::AsField, UtxoId};
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{
    packets::{PublishError, PublishOpts, PublishPacket},
    FuelCoreLike,
};

pub fn publish_tasks(
    tx: &Transaction,
    tx_id: &Bytes32,
    stream: &Stream<Utxo>,
    opts: &Arc<PublishOpts>,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let packets: Vec<(UtxosSubject, Utxo)> = tx
        .inputs()
        .par_iter()
        .filter_map(|input| {
            find_utxo(
                input,
                tx_id.to_owned(),
                input.utxo_id().cloned(),
                fuel_core,
            )
        })
        .collect();

    packets
        .into_iter()
        .map(|(subject, utxo)| {
            let packet = PublishPacket::new(
                &utxo,
                subject.arc(),
                UtxosSubject::WILDCARD,
            );
            packet.publish(Arc::new(stream.to_owned()), opts)
        })
        .collect()
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

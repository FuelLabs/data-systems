mod blocks;
mod inputs;
mod logs;
mod outputs;
mod receipts;
mod transactions;
mod utxos;

mod fuel_core;
mod publisher;

pub mod cli;
pub mod metrics;
pub mod server;
pub mod shutdown;
pub mod state;
pub mod system;

use std::sync::Arc;

pub use fuel_core::{FuelCore, FuelCoreLike};
use fuel_streams_core::prelude::*;
use metrics::PublisherMetrics;
pub use publisher::{Publisher, Streams};
use sha2::{Digest, Sha256};

pub fn sha256(bytes: &[u8]) -> Bytes32 {
    let mut sha256 = Sha256::new();
    sha256.update(bytes);
    let bytes: [u8; 32] = sha256
        .finalize()
        .as_slice()
        .try_into()
        .expect("Must be 32 bytes");

    bytes.into()
}

pub fn maybe_include_predicate_and_script_subjects(
    subjects: &mut Vec<(Box<dyn IntoSubject>, &'static str)>,
    predicate_tag: &Option<Bytes32>,
    script_tag: &Option<Bytes32>,
) {
    if let Some(predicate_tag) = predicate_tag.clone() {
        subjects.push((
            InputsByIdSubject::new()
                .with_id_kind(Some(IdentifierKind::PredicateID))
                .with_id_value(Some(predicate_tag))
                .boxed(),
            InputsByIdSubject::WILDCARD,
        ));
    }

    if let Some(script_tag) = script_tag.clone() {
        subjects.push((
            InputsByIdSubject::new()
                .with_id_kind(Some(IdentifierKind::ScriptID))
                .with_id_value(Some(script_tag))
                .boxed(),
            InputsByIdSubject::WILDCARD,
        ));
    }
}

pub async fn publish_all<S: Streamable>(
    stream: &Stream<S>,
    subjects: Vec<(Box<dyn IntoSubject>, &'static str)>,
    payload: &S,
    metrics: &Arc<PublisherMetrics>,
    chain_id: &ChainId,
    block_producer: &Address,
) {
    for (subject, wildcard) in subjects {
        publish_with_metrics!(
            stream.publish(&*subject, payload),
            metrics,
            chain_id,
            block_producer,
            wildcard
        );
    }
}

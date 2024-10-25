mod blocks;
mod identifiers;
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

use std::{
    env,
    sync::{Arc, LazyLock},
};

pub use fuel_core::{FuelCore, FuelCoreLike};
use fuel_streams_core::prelude::*;
use metrics::PublisherMetrics;
pub use publisher::{Publisher, Streams};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub static CONCURRENCY_LIMIT: LazyLock<usize> = LazyLock::new(|| {
    env::var("CONCURRENCY_LIMIT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(32)
});

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

pub type SubjectPayload = (Box<dyn IntoSubject>, &'static str);

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Failed to publish to stream: {0}")]
    StreamPublishError(String), // Customize the error message as needed

    #[error("Semaphore acquisition failed: {0}")]
    SemaphoreError(#[from] tokio::sync::AcquireError),

    // Add other error variants as needed
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub struct PublishPayload<S: Streamable> {
    pub stream: Stream<S>,
    pub subject: SubjectPayload,
    pub payload: S,
}

impl<T: Streamable + Clone + Send + Sync + 'static> PublishPayload<T> {
    pub async fn publish(
        &self,
        metrics: &Arc<PublisherMetrics>,
        chain_id: &ChainId,
        block_producer: &Address,
    ) -> Result<(), PublishError> {
        let (subject, wildcard) = &self.subject;
        let wildcard = *wildcard;
        publish_with_metrics!(
            self.stream.publish(&**subject, &self.payload),
            metrics,
            chain_id,
            block_producer,
            wildcard
        )
    }
}

mod blocks;
mod inputs;
mod logs;
mod outputs;
mod receipts;
mod transactions;
mod utxos;

mod fuel_core;
mod packets;
mod publisher;

pub mod cli;
pub mod identifiers;
pub mod elastic;
pub mod metrics;
pub mod server;
pub mod shutdown;
pub mod state;
pub mod system;

use std::{env, sync::LazyLock};

use elastic::ElasticSearch;
use elasticsearch::params::Refresh;
pub use fuel_core::{FuelCore, FuelCoreLike};
use fuel_streams_core::prelude::*;
pub use publisher::{Publisher, Streams};
use sha2::{Digest, Sha256};


pub static CONCURRENCY_LIMIT: LazyLock<usize> = LazyLock::new(|| {
    env::var("CONCURRENCY_LIMIT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(32)
});

pub const FUEL_ELASTICSEARCH_PATH: &str = "fuel-data-systems";

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

pub async fn log_all<S: Streamable>(
    elastic_logger: &Option<Arc<ElasticSearch>>,
    subjects: &[(Box<dyn IntoSubject>, &'static str)],
    payload: &S,
) {
    if let Some(elastic_logger) = elastic_logger.as_ref() {
        for (subject, _wildcard) in subjects {
            let id = &subject.parse();
            if let Err(e) = elastic_logger
                .get_conn()
                .index(
                    FUEL_ELASTICSEARCH_PATH,
                    Some(id),
                    payload,
                    Some(Refresh::WaitFor),
                )
                .await
            {
                tracing::error!("Failed to log to ElasticSearch: {:?}", e);
            }
        }
    }
}

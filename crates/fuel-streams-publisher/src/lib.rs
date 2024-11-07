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
pub mod elastic;
pub mod identifiers;
pub mod metrics;
pub mod server;
pub mod shutdown;
pub mod state;
pub mod system;

use std::{env, sync::LazyLock};

pub use fuel_core::{FuelCore, FuelCoreLike};
use fuel_streams_core::prelude::*;
pub use publisher::{Publisher, Streams};
use sha2::{Digest, Sha256};

pub static PUBLISHER_MAX_THREADS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = (available_cpus / 3).max(1); // Use 1/3 of CPUs, minimum 1

    env::var("PUBLISHER_MAX_THREADS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
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

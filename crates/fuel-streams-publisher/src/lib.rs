pub mod cli;
pub mod grpc;
pub mod publisher;
pub mod server;
pub mod telemetry;

use std::{env, sync::LazyLock};

use fuel_streams_core::prelude::*;
pub use publisher::{
    core::{Publisher, Streams},
    fuel::{FuelCore, FuelCoreLike},
};
use sha2::{Digest, Sha256};

pub static PUBLISHER_MAX_THREADS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = (available_cpus / 3).max(1); // Use 1/3 of CPUs, minimum 1

    env::var("PUBLISHER_MAX_THREADS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
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

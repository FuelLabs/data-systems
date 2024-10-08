mod blocks;
pub mod cli;
mod inputs;
mod logs;
mod outputs;
mod publisher;
mod receipts;
mod transactions;
mod utxos;

mod fuel_core;

pub mod metrics;
pub mod server;
pub mod shutdown;
pub mod state;
pub mod system;

pub use fuel_core::{FuelCore, FuelCoreLike};
use fuel_streams_core::prelude::*;
pub use publisher::{Publisher, Streams};
use sha2::{Digest, Sha256};

fn prefix_subject(
    prefix: &Option<String>,
    subject: &dyn IntoSubject,
) -> String {
    let subject = subject.parse();
    match prefix {
        Some(prefix) => format!("{prefix}.{subject}"),
        None => subject,
    }
}

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

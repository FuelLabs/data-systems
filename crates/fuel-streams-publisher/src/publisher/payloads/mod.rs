pub mod blocks;
pub mod identifiers;
pub mod inputs;
pub mod logs;
pub mod outputs;
pub mod receipts;
pub mod transactions;
pub mod utxos;

use fuel_streams_core::prelude::Bytes32;
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

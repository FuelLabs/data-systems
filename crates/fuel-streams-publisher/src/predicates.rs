use fuel_streams_core::types::Bytes32;
use sha2::{Digest, Sha256};

pub fn tag(bytecode: &[u8]) -> Bytes32 {
    let mut sha256 = Sha256::new();
    sha256.update(bytecode);
    let bytes: [u8; 32] = sha256
        .finalize()
        .as_slice()
        .try_into()
        .expect("Must be 32 bytes");

    bytes.into()
}

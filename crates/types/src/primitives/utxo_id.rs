use std::{fmt, str::FromStr};

use apache_avro::AvroSchema;
use rand::Rng;

use super::Bytes32;
use crate::fuel_core::*;

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
    AvroSchema,
)]
pub struct UtxoId {
    pub tx_id: Bytes32,
    pub output_index: u16,
}

impl UtxoId {
    pub fn random() -> Self {
        Self {
            tx_id: Bytes32::random(),
            output_index: rand::rng().random_range(0..u16::MAX),
        }
    }
}

impl fmt::Display for UtxoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fuel_core_types::fuel_types::canonical::Deserialize;
        let tx_id_bytes = self.tx_id.to_owned().into_inner();
        let tx_id_bytes = tx_id_bytes.as_slice();
        let tx_id_bytes = FuelCoreBytes32::from_bytes(tx_id_bytes)
            .expect("Tx ID not valid to convert for bytes");
        let utxo_id = FuelCoreUtxoId::new(tx_id_bytes, self.output_index);
        write!(f, "0x{utxo_id}")
    }
}

impl From<FuelCoreUtxoId> for UtxoId {
    fn from(value: FuelCoreUtxoId) -> Self {
        Self::from(&value)
    }
}

impl From<&FuelCoreUtxoId> for UtxoId {
    fn from(value: &FuelCoreUtxoId) -> Self {
        Self {
            tx_id: value.tx_id().into(),
            output_index: value.output_index(),
        }
    }
}

impl FromStr for UtxoId {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let utxo_id = FuelCoreUtxoId::from_str(s)?;
        Ok(Self::from(&utxo_id))
    }
}

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
    derive_more::Display,
)]
#[display("{}", self.tx_id.to_string())]
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

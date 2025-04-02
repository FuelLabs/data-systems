use rand::Rng;

use super::BlockHeight;
use crate::fuel_core::*;

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
    utoipa::ToSchema,
)]
pub struct TxPointer {
    block_height: BlockHeight,
    tx_index: u16,
}

impl TxPointer {
    pub fn random() -> Self {
        Self {
            block_height: BlockHeight::random(),
            tx_index: rand::rng().random_range(0..u16::MAX),
        }
    }
}

impl From<FuelCoreTxPointer> for TxPointer {
    fn from(value: FuelCoreTxPointer) -> Self {
        Self {
            block_height: value.block_height().into(),
            tx_index: value.tx_index(),
        }
    }
}

impl From<Option<FuelCoreTxPointer>> for TxPointer {
    fn from(value: Option<FuelCoreTxPointer>) -> Self {
        value.unwrap_or_default().into()
    }
}

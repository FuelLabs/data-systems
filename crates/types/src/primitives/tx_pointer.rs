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
)]
pub struct TxPointer {
    block_height: FuelCoreBlockHeight,
    tx_index: u16,
}

impl From<FuelCoreTxPointer> for TxPointer {
    fn from(value: FuelCoreTxPointer) -> Self {
        Self {
            block_height: value.block_height(),
            tx_index: value.tx_index(),
        }
    }
}

impl From<Option<FuelCoreTxPointer>> for TxPointer {
    fn from(value: Option<FuelCoreTxPointer>) -> Self {
        value.unwrap_or_default().into()
    }
}
